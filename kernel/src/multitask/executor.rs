// The MIT License (MIT)
//
// Copyright (c) 2019 Philipp Oppermann
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.


use super::task;
use alloc::{collections::BTreeMap, rc::Rc};
use core::{
    cell::RefCell,
    task::{Context, Poll, Waker},
};
use x86_64::instructions::interrupts;

pub struct Executor {
    task_collection: Rc<RefCell<task::Collection>>,
    waker_collection: BTreeMap<task::Id, Waker>,
}

impl Executor {
    pub fn new(task_collection: Rc<RefCell<task::Collection>>) -> Self {
        Self {
            task_collection,
            waker_collection: BTreeMap::new(),
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_woken_tasks();
            self.sleep_if_idle();
        }
    }

    fn sleep_if_idle(&self) {
        interrupts::disable();
        if self.task_collection.borrow().woken_task_exists() {
            interrupts::enable()
        } else {
            interrupts::enable_and_hlt()
        }
    }

    fn run_woken_tasks(&mut self) {
        while let Some(id) = self.pop_woken_task_id() {
            self.run_task(id);
        }
    }

    fn pop_woken_task_id(&mut self) -> Option<task::Id> {
        self.task_collection.borrow_mut().pop_woken_task_id()
    }

    fn run_task(&mut self, id: task::Id) {
        let Self {
            task_collection,
            waker_collection,
        } = self;

        let mut task = match task_collection.borrow_mut().remove_task(id) {
            Some(task) => task,
            None => return,
        };

        let waker = waker_collection
            .entry(id)
            .or_insert_with(|| task_collection.borrow_mut().create_waker(id));

        let mut context = Context::from_waker(waker);
        match task.poll(&mut context) {
            Poll::Ready(_) => {
                self.task_collection.borrow_mut().remove_task(id);
                self.waker_collection.remove(&id);
            }
            Poll::Pending => self.task_collection.borrow_mut().add_task_as_sleep(task),
        }
    }
}
