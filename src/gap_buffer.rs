use std::io::{Error, ErrorKind};
use std::{ptr::NonNull, alloc::Layout};

use std::{alloc, ptr};

pub struct GapWindow {
    pub index: usize,
    pub window_size: usize
}

pub struct TextGapBuffer {
    window_size: usize,

    ptr: NonNull<char>,
    gap_window: GapWindow,
    size: usize,
    cap: usize
}

impl TextGapBuffer {
    pub fn new () -> Self {
        TextGapBuffer::new_in(1200, 100)
    }

    pub fn new_in (initial_cap: usize, window_size: usize) -> Self {
        let layout = match Layout::array::<char>(initial_cap) {
            Ok(layout) => layout,
            Err(_) => panic!("Error")
        };

        let ptr = unsafe { alloc::alloc(layout) };
        let ptr = match NonNull::new(ptr as *mut char) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(layout)
        };

        let gap_window = GapWindow {
            index: 0,
            window_size
        };

        TextGapBuffer {
            window_size,

            ptr,
            gap_window,
            size: 0,
            cap: initial_cap
        }
    }

    fn grow (self: &mut Self) {
        let cap = 2 * self.cap;
        let layout = match Layout::array::<char>(cap) {
            Ok(layout) => layout,
            Err(_) => panic!("Error")
        };

        let ptr = unsafe {
            alloc::realloc(
                self.ptr.as_ptr() as *mut u8,
                Layout::array::<char>(self.cap).unwrap(),
                layout.size()
            )
        };

        self.ptr = match NonNull::new(ptr as *mut char) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(layout)
        };

        self.cap = cap;
    }

    pub fn move_window (self: &mut Self, pos: usize) {
        let pos = if pos > self.size { self.size } else { pos };
        let window_end_index = self.gap_window.index + self.gap_window.window_size;

        if pos < self.gap_window.index {
            let move_size = self.gap_window.index - pos;
            unsafe {
                std::ptr::copy(
                    self.ptr.as_ptr().add(pos),
                    self.ptr.as_ptr().add(window_end_index - move_size),
                    move_size
                );
            }
        }

        if pos > self.gap_window.index {
            let move_size = (pos + self.gap_window.window_size) - window_end_index;
            unsafe {
                std::ptr::copy(
                    self.ptr.as_ptr().add(window_end_index),
                    self.ptr.as_ptr().add(self.gap_window.index),
                    move_size
                );
            }
        }

        self.gap_window.index = pos;
    }

    fn resize_window (self: &mut Self) {
        if self.cap <= self.size + self.window_size {
            self.grow();
        }

        let full_size = self.size + self.window_size;
        let size = full_size - self.gap_window.index;
        unsafe {
            std::ptr::copy(
                self.ptr.as_ptr().add(self.gap_window.index),
                self.ptr.as_ptr().add(self.gap_window.index + self.window_size),
                size
            );
        }

        self.gap_window.window_size = self.window_size;
    }

    pub fn insert_ch (self: &mut Self, ch: char) {
        if self.cap <= self.size + self.gap_window.window_size {
            self.grow();
        }

        unsafe {
            ptr::write(self.ptr.as_ptr().add(self.gap_window.index), ch);
        }

        self.gap_window.index += 1;
        self.gap_window.window_size -= 1;
        self.size += 1;

        if self.gap_window.window_size == 0 {
            self.resize_window();
        }
    }

    pub fn delete_ch (self: &mut Self) {
        self.gap_window.index -= 1;
        self.gap_window.window_size += 1;
        self.size -= 1;
    }

    pub fn len (self: &Self) -> usize {
        self.size
    }

    pub fn get (self: &Self, relative_index: usize) -> Result<char, Error> {
        let index = self.get_absolute_index(relative_index)?;
        unsafe {
            Ok(self.ptr.as_ptr().add(index).read())
        }
    }

    //Gets you the relative index of the string
    fn get_relative_index (self: &Self, absolute_index: usize) -> Result<usize, Error> {
        if absolute_index >= self.size + self.gap_window.window_size {
            return Err(Error::new(ErrorKind::OutOfMemory, "get_relative_index: Index out of bound"))
        }

        let end_window_index = self.gap_window.index + self.gap_window.window_size - 1;
        if absolute_index > end_window_index {
            Ok(absolute_index - self.gap_window.window_size)
        } else {
            Ok(absolute_index)
        }
    }

    //Gets you the absolute index of the array with the gap
    fn get_absolute_index (self: &Self, relative_index: usize) -> Result<usize, Error> {
        if relative_index >= self.size {
            return Err(Error::new(ErrorKind::OutOfMemory, "get_absolute_index: Index out of bound"))
        }

        if relative_index >= self.gap_window.index {
            Ok(relative_index + self.gap_window.window_size)
        } else {
            Ok(relative_index)
        }
    }
}

impl Drop for TextGapBuffer {
    fn drop (self: &mut Self) {
        if self.cap != 0 {
            println!("==> Freeing TextGapBuffer");
            let layout = Layout::array::<char>(self.cap).unwrap();
            unsafe {
                alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
            }
            println!("==> Freed TextGapBuffer");
        }
    }
}
