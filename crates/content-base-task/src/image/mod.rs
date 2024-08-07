pub mod desc_embed;
pub mod description;
pub mod thumbnail;

use content_base_derive::ContentTask;
use desc_embed::ImageDescEmbedTask;
use description::ImageDescriptionTask;
use storage_macro::Storage;
use strum::{EnumIter, EnumString};
use thumbnail::ImageThumbnailTask;

use crate::ContentTaskType;

#[derive(Clone, Debug, EnumIter, EnumString, strum_macros::Display, ContentTask, Storage)]
#[strum(serialize_all = "kebab-case")]
pub enum ImageTaskType {
    Thumbnail(ImageThumbnailTask),
    Description(ImageDescriptionTask),
    DescEmbed(ImageDescEmbedTask),
}

impl Into<ContentTaskType> for ImageTaskType {
    fn into(self) -> ContentTaskType {
        ContentTaskType::Image(self)
    }
}
