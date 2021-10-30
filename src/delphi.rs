//! Handles Lazarus lpi files
extern crate xml;

use std::io::{Read, Write};
use std::path::PathBuf;

use crate::files::{ContentProcessor, FileFinder, RootFileFinderByExt};
use crate::sem_ver::{SemVer, SemVerComponent, SemVerComponentSet};
use crate::xml_util::{echo, transform_xml, ElementPath, XmlError};
use xml::attribute::OwnedAttribute;
use xml::name::OwnedName;
use xml::namespace::Namespace;
use xml::reader::{EventReader, XmlEvent};
use xml::writer::EventWriter;

/// Handles versioning found in Lazarus lpi files.
pub struct LpiUpdater {}

impl FileFinder for LpiUpdater {
    fn find(&self, dir: &str) -> std::io::Result<Vec<PathBuf>> {
        RootFileFinderByExt::new("lpi").find(dir)
    }
}

impl ContentProcessor for LpiUpdater {
    type Err = XmlError;

    fn process(&self, old_contents: &str, version: SemVer) -> Result<String, Self::Err> {
        process_str(old_contents, version)
    }
}

fn process_str(old_contents: &str, version: SemVer) -> Result<String, XmlError> {
    transform_xml(old_contents, |parser, writer| {
        do_process(parser, writer, version)
    })
}

fn do_process<R: Read, W: Write>(
    parser: EventReader<R>,
    writer: &mut EventWriter<W>,
    version: SemVer,
) -> Result<(), XmlError> {
    let mut element_path = ElementPath::Empty;
    let mut found_sem_ver_components = SemVerComponentSet::new();
    for result_xml_event in parser {
        let xml_event = result_xml_event?;
        match &xml_event {
            XmlEvent::StartElement {
                name,
                attributes,
                namespace,
            } => {
                element_path = element_path.push(&name.local_name);
                match match_sem_ver_element(&element_path) {
                    Some(sem_ver_component) => {
                        found_sem_ver_components += sem_ver_component;
                        let value_as_str = version.get_component(sem_ver_component).to_string();
                        writer.write(add_or_update_attribute(
                            name,
                            attributes,
                            namespace,
                            "Value",
                            &value_as_str,
                        ))?;
                    }
                    _ => {
                        echo(&xml_event, writer)?;
                    }
                }
            }
            XmlEvent::EndElement { .. } => {
                let is_popping_version_info =
                    element_path.matches(&["CONFIG", "ProjectOptions", "VersionInfo"]);
                element_path = element_path.pop();
                if is_popping_version_info {
                    for missing in found_sem_ver_components.missing() {
                        let name = sem_ver_component_to_element_name(missing);
                        let value_as_str = version.get_component(missing).to_string();
                        writer.write(
                            xml::writer::XmlEvent::start_element(name).attr("Value", &value_as_str),
                        )?;
                        writer.write(xml::writer::XmlEvent::end_element())?;
                    }
                }
                echo(&xml_event, writer)?;
            }
            XmlEvent::Whitespace(_) => {
                // discarding whitespace because it confuses indentation
            }
            _ => {
                echo(&xml_event, writer)?;
            }
        }
    }
    Ok(())
}

fn add_or_update_attribute<'a>(
    name: &'a OwnedName,
    attributes: &'a Vec<OwnedAttribute>,
    _namespace: &'a Namespace,
    attr_name: &'a str,
    value: &'a str,
) -> xml::writer::XmlEvent<'a> {
    let mut builder = xml::writer::XmlEvent::start_element(name.borrow());
    let mut found = false;
    for attribute in attributes {
        if !found && attribute.name.local_name == attr_name {
            found = true;
            builder = builder.attr(attribute.name.borrow(), value);
        } else {
            builder = builder.attr(attribute.name.borrow(), &attribute.value);
        }
    }
    if !found {
        builder = builder.attr(attr_name, value);
    }
    builder.into()
}

fn match_sem_ver_element(element_path: &ElementPath) -> Option<SemVerComponent> {
    SemVerComponentSet::all()
        .filter(|component| {
            element_path.matches(&[
                "CONFIG",
                "ProjectOptions",
                "VersionInfo",
                sem_ver_component_to_element_name(*component),
            ])
        })
        .next()
}

fn sem_ver_component_to_element_name(component: SemVerComponent) -> &'static str {
    match component {
        SemVerComponent::Major => "MajorVersionNr",
        SemVerComponent::Minor => "MinorVersionNr",
        SemVerComponent::Patch => "RevisionNr",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_elements_present() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<CONFIG>
  <ProjectOptions>
    <Version Value="11"/>
    <i18n>
      <EnableI18N LFM="False"/>
    </i18n>
    <VersionInfo>
      <UseVersionInfo Value="True"/>
      <AutoIncrementBuild Value="True"/>
      <MajorVersionNr Value="1"/>
      <MinorVersionNr Value="1"/>
      <RevisionNr Value="2"/>
      <BuildNr Value="2"/>
    </VersionInfo>
  </ProjectOptions>
</CONFIG>
    "#;
        let expected = r#"<?xml version="1.0" encoding="UTF-8"?>
<CONFIG>
  <ProjectOptions>
    <Version Value="11" />
    <i18n>
      <EnableI18N LFM="False" />
    </i18n>
    <VersionInfo>
      <UseVersionInfo Value="True" />
      <AutoIncrementBuild Value="True" />
      <MajorVersionNr Value="3" />
      <MinorVersionNr Value="4" />
      <RevisionNr Value="5" />
      <BuildNr Value="2" />
    </VersionInfo>
  </ProjectOptions>
</CONFIG>
"#;
        let result = process_str(input, SemVer::new(3, 4, 5)).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn adds_missing_value_attribute() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<CONFIG>
  <ProjectOptions>
    <VersionInfo>
      <MajorVersionNr Oops="1"/>
    </VersionInfo>
  </ProjectOptions>
</CONFIG>
    "#;
        let expected = r#"<?xml version="1.0" encoding="UTF-8"?>
<CONFIG>
  <ProjectOptions>
    <VersionInfo>
      <MajorVersionNr Oops="1" Value="2" />
      <MinorVersionNr Value="3" />
      <RevisionNr Value="4" />
    </VersionInfo>
  </ProjectOptions>
</CONFIG>
"#;
        let result = process_str(input, SemVer::new(2, 3, 4)).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn does_not_affect_elements_outside_version_info() {
        let input = r#"<?xml version="1.0" encoding="UTF-8"?>
<CONFIG>
  <ProjectOptions>
    <MajorVersionNr Oops="1" />
  </ProjectOptions>
</CONFIG>
"#;
        let result = process_str(input, SemVer::new(2, 3, 4)).unwrap();
        assert_eq!(result, input);
    }
}
