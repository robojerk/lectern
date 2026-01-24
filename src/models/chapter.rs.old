use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use std::cell::{Cell, RefCell};

mod imp {
    use super::*;
    use gtk4::glib::{ParamSpec, ParamSpecString, ParamSpecUInt64, ParamSpecBoolean, Value};
    use once_cell::sync::Lazy;

    #[derive(Default)]
    pub struct ChapterObject {
        pub title: RefCell<String>,
        pub start_time: Cell<u64>,
        pub duration: Cell<u64>,
        pub is_locked: Cell<bool>,
    }

    #[gtk4::glib::object_subclass]
    impl ObjectSubclass for ChapterObject {
        const NAME: &'static str = "ChapterObject";
        type Type = super::ChapterObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for ChapterObject {
        fn properties() -> &'static [ParamSpec] {
            static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
                vec![
                    ParamSpecString::builder("title").build(),
                    ParamSpecUInt64::builder("start-time").build(),
                    ParamSpecUInt64::builder("duration").build(),
                    ParamSpecBoolean::builder("is-locked").build(),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(&self, _id: usize, _value: &Value, pspec: &ParamSpec) {
            match pspec.name() {
                "title" => {
                    let input_value = _value.get().expect("The value needs to be of type `String`.");
                    self.title.replace(input_value);
                },
                "start-time" => {
                    let input_value = _value.get().expect("The value needs to be of type `u64`.");
                    self.start_time.set(input_value);
                },
                "duration" => {
                    let input_value = _value.get().expect("The value needs to be of type `u64`.");
                    self.duration.set(input_value);
                },
                "is-locked" => {
                     let input_value = _value.get().expect("The value needs to be of type `bool`.");
                     self.is_locked.set(input_value);
                },
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &ParamSpec) -> Value {
            match pspec.name() {
                "title" => self.title.borrow().to_value(),
                "start-time" => self.start_time.get().to_value(),
                "duration" => self.duration.get().to_value(),
                "is-locked" => self.is_locked.get().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct ChapterObject(ObjectSubclass<imp::ChapterObject>);
}

impl ChapterObject {
    pub fn new(title: String, start_time: u64, duration: u64) -> Self {
        glib::Object::builder()
            .property("title", title)
            .property("start-time", start_time)
            .property("duration", duration)
            .build()
    }
    
    pub fn title(&self) -> String {
        self.property("title")
    }

    pub fn set_title(&self, title: String) {
        self.set_property("title", title);
    }

    pub fn start_time(&self) -> u64 {
        self.property("start-time")
    }

    pub fn set_start_time(&self, time: u64) {
        self.set_property("start-time", time);
    }

    pub fn duration(&self) -> u64 {
        self.property("duration")
    }

    pub fn set_duration(&self, duration: u64) {
        self.set_property("duration", duration);
    }

    pub fn is_locked(&self) -> bool {
        self.property("is-locked")
    }

    pub fn set_locked(&self, locked: bool) {
        self.set_property("is-locked", locked);
    }
}
