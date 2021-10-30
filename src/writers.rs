use crate::git;
use std::path::PathBuf;

pub trait FileWriter {
    fn write(&self, path: &PathBuf, contents: &str) -> std::io::Result<()>;

    fn compose<B: FileWriter>(self, other: B) -> CompositeWriter<Self, B>
    where
        Self: Sized,
    {
        CompositeWriter::new(self, other)
    }
}

pub fn create_writer(git_dir: PathBuf, dry_run: bool) -> Box<dyn FileWriter> {
    if dry_run {
        Box::new(DryFileWriter {})
    } else {
        Box::new(WetFileWriter {}.compose(GitAddWriter { git_dir }))
    }
}

struct DryFileWriter {}

impl FileWriter for DryFileWriter {
    fn write(&self, path: &PathBuf, _contents: &str) -> std::io::Result<()> {
        println!("Would have written {}", path.to_string_lossy());
        Ok(())
    }
}

struct WetFileWriter {}

impl FileWriter for WetFileWriter {
    fn write(&self, path: &PathBuf, contents: &str) -> std::io::Result<()> {
        std::fs::write(path, contents)
    }
}

struct GitAddWriter {
    git_dir: PathBuf,
}

impl FileWriter for GitAddWriter {
    fn write(&self, path: &PathBuf, _contents: &str) -> std::io::Result<()> {
        match path.strip_prefix(&self.git_dir) {
            Ok(item_to_add) => match git::add(&self.git_dir, item_to_add) {
                Ok(_) => Ok(()),
                Err(err) => Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
            },
            Err(err) => Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
        }
    }
}

pub struct CompositeWriter<A, B> {
    first: A,
    second: B,
}

impl<A: FileWriter, B: FileWriter> CompositeWriter<A, B> {
    pub fn new(first: A, second: B) -> Self {
        Self { first, second }
    }
}

impl<A: FileWriter, B: FileWriter> FileWriter for CompositeWriter<A, B> {
    fn write(&self, path: &PathBuf, contents: &str) -> std::io::Result<()> {
        self.first
            .write(path, contents)
            .and_then(|_| self.second.write(path, contents))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// Dummy file writer
    struct DummyWriter {
        /// Inner mutable tracking of whether this writer was called or not
        called: Rc<RefCell<bool>>,

        /// should the writer fail or not
        should_fail: bool,
    }

    impl DummyWriter {
        pub fn new(called: Rc<RefCell<bool>>, should_fail: bool) -> Self {
            Self {
                called,
                should_fail,
            }
        }

        fn mark_called(&self) {
            *self.called.borrow_mut() = true;
        }
    }

    impl FileWriter for DummyWriter {
        fn write(&self, _path: &PathBuf, _contents: &str) -> std::io::Result<()> {
            self.mark_called();
            if self.should_fail {
                Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe))
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn composite_writer_both_succeed() {
        // arrange
        let first_called = Rc::new(RefCell::new(false));
        let second_called = Rc::new(RefCell::new(false));
        let first = DummyWriter::new(Rc::clone(&first_called), false);
        let second = DummyWriter::new(Rc::clone(&second_called), false);
        let composite = first.compose(second);
        let path_buf = PathBuf::new();
        let contents = "";

        // act and assert
        composite
            .write(&path_buf, contents)
            .expect("composite should succeed");

        // assert
        assert!(first_called.replace(false), "first writer should be called");
        assert!(
            second_called.replace(false),
            "second writer should be called"
        );
    }

    #[test]
    fn composite_writer_second_fails() {
        // arrange
        let first_called = Rc::new(RefCell::new(false));
        let second_called = Rc::new(RefCell::new(false));
        let first = DummyWriter::new(Rc::clone(&first_called), false);
        let second = DummyWriter::new(Rc::clone(&second_called), true);
        let composite = first.compose(second);
        let path_buf = PathBuf::new();
        let contents = "";

        // act and assert
        composite
            .write(&path_buf, contents)
            .expect_err("composite should fail");

        // assert
        assert!(first_called.replace(false), "first writer should be called");
        assert!(
            second_called.replace(false),
            "second writer should be called"
        );
    }

    #[test]
    fn composite_writer_first_fails() {
        // arrange
        let first_called = Rc::new(RefCell::new(false));
        let second_called = Rc::new(RefCell::new(false));
        let first = DummyWriter::new(Rc::clone(&first_called), true);
        let second = DummyWriter::new(Rc::clone(&second_called), false);
        let composite = first.compose(second);
        let path_buf = PathBuf::new();
        let contents = "";

        // act and assert
        composite
            .write(&path_buf, contents)
            .expect_err("composite should fail");
        assert!(first_called.replace(false), "first writer should be called");
        assert!(
            !second_called.replace(false),
            "second writer should not have been called"
        );
    }
}
