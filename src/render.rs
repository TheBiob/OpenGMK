//! Game rendering functionality

pub mod opengl;

use crate::atlas::AtlasBuilder;
use std::{io, path::PathBuf};

pub trait Renderer {
    /// Stores & uploads atlases to the GPU.
    /// This function is for initializing, and should be called only once.
    ///
    /// Returns a handle to each inserted texture (in insertion order).
    fn upload_atlases(&mut self, atl: AtlasBuilder) -> Result<(), String>;

    /// Dumps atlases to filepaths provided by `Fn(index: usize) -> PathBuf`.
    fn dump_atlases(&self, path: impl Fn(usize) -> PathBuf) -> io::Result<()>;

    /// Returns the max texture size the GPU can hold.
    fn max_gpu_texture_size(&self) -> usize;

    /// Indicates whether the window wants to close.
    fn should_close(&self) -> bool;

    /// Draws a sprite to the screen. Parameters are similar to those of GML's draw_sprite_ext.
    fn draw_sprite(
        &self,
        texture: &Texture,
        x: f64,
        y: f64,
        xscale: f64,
        yscale: f64,
        angle: f64,
        colour: i32,
        alpha: f64,
    );

    /// Updates the screen with new drawings for the current frame.
    fn draw(&mut self);
}

pub struct RendererOptions<'a> {
    pub title: &'a str,
    pub size: (u32, u32),
    pub icon: Option<(Vec<u8>, u32, u32)>,
    pub resizable: bool,
    pub on_top: bool,
    pub decorations: bool,
    pub fullscreen: bool,
    pub vsync: bool,
}

pub struct Texture(usize);

impl From<usize> for Texture {
    fn from(n: usize) -> Self {
        Texture(n)
    }
}
