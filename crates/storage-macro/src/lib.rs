use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Storage)]
pub fn storage_trait_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        use storage::prelude::*;

        impl #name {
            fn storage(&self) -> StorageResult<impl Storage> {
                global_variable::get_current_fs_storage!()
            }
        }

        #[async_trait::async_trait]
        impl Storage for #name {
            fn clone_box(&self) -> Box<dyn Storage> {
                self.storage().unwrap().clone_box()
            }

            fn root(&self) -> StorageResult<std::path::PathBuf> {
                self.storage()?.root()
            }

            fn op(&self) -> StorageResult<storage::Operator> {
                self.storage()?.op()
            }

            fn block_op(&self) -> StorageResult<storage::BlockingOperator> {
                self.storage().and_then(|s| s.block_op())
            }

            fn get_absolute_path(&self, relative_path: std::path::PathBuf) -> StorageResult<std::path::PathBuf> {
                self.storage()?.get_absolute_path(relative_path)
            }

            fn under_root(&self, path: std::path::PathBuf) -> StorageResult<bool> {
                self.storage()?.under_root(path)
            }

            fn read_blocking(&self, path: std::path::PathBuf) -> StorageResult<storage::Buffer> {
                self.storage()?.read_blocking(path)
            }

            fn read_to_string(&self, path: std::path::PathBuf) -> StorageResult<String> {
                self.storage()?.read_to_string(path)
            }

            fn write_blocking(&self, path: std::path::PathBuf, bs: storage::Bytes) -> StorageResult<()> {
                self.storage()?.write_blocking(path, bs)
            }

            fn remove_file(&self, path: std::path::PathBuf) -> StorageResult<()> {
                self.storage()?.remove_file(path)
            }

            async fn create_dir(&self, path: std::path::PathBuf) -> StorageResult<()> {
                self.storage()?.create_dir(path).await
            }

            async fn is_exist(&self, path: std::path::PathBuf) -> StorageResult<bool> {
                self.storage()?.is_exist(path).await
            }

            async fn read(&self, path: std::path::PathBuf) -> StorageResult<storage::Buffer> {
                self.storage()?.read(path).await
            }

            async fn write(&self, path: std::path::PathBuf, bs: storage::Buffer) -> StorageResult<()> {
                self.storage()?.write(path, bs).await
            }

            async fn copy(&self, from: std::path::PathBuf, to: std::path::PathBuf) -> StorageResult<()> {
                self.storage()?.copy(from, to).await
            }

            async fn read_dir(&self, path: std::path::PathBuf) -> StorageResult<Vec<std::path::PathBuf>> {
                self.storage()?.read_dir(path).await
            }

            async fn remove_dir_all(&self, path: std::path::PathBuf) -> StorageResult<()> {
                self.storage()?.remove_dir_all(path).await
            }

            async fn len(&self, path: std::path::PathBuf) -> StorageResult<u64> {
                self.storage()?.len(path).await
            }

            async fn upload_dir_recursive(
                &self,
                dir: std::path::PathBuf,
            ) -> StorageResult<()> {
                self.storage()?.upload_dir_recursive(dir).await
            }

            async fn read_with_range(
                &self,
                path: std::path::PathBuf,
                range: std::ops::Range<u64>,
            ) -> StorageResult<storage::Buffer> {
                self.storage()?.read_with_range(path, range).await
            }
        }
    };

    TokenStream::from(expanded)
}
