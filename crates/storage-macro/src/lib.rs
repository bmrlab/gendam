use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(StorageTrait)]
pub fn storage_trait_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = quote! {
        use global_variable::get_current_storage;
        use storage::StorageError;
        use storage::StorageResult;
        use storage::Bytes;
        use storage::Buffer;
        use storage::Storage;
        use storage::StorageTrait;

        #[async_trait]
        impl StorageTrait for #name {
            fn get_storage(&self) -> StorageResult<Storage> {
                get_current_storage!()
            }

            fn get_actual_path(&self, path: std::path::PathBuf) -> StorageResult<std::path::PathBuf> {
                std::result::Result::Ok(self.get_storage()?.get_actual_path(path))
            }

            fn read_blocking(&self, path: std::path::PathBuf) -> StorageResult<storage::Buffer> {
                self.get_storage()?.read_blocking(path)
            }

            fn read_to_string(&self, path: std::path::PathBuf) -> StorageResult<String> {
                self.get_storage()?.read_to_string(path)
            }

            fn write_blocking(&self, path: std::path::PathBuf, bs: Bytes) -> StorageResult<()> {
                self.get_storage()?.write_blocking(path, bs)
            }

            fn remove_file(&self, path: std::path::PathBuf) -> StorageResult<()> {
                self.get_storage()?.remove_file(path)
            }

            async fn create_dir(&self, path: std::path::PathBuf) -> StorageResult<()> {
                self.get_storage()?.create_dir(path).await
            }

            async fn is_exist(&self, path: std::path::PathBuf) -> StorageResult<bool> {
                self.get_storage()?.is_exist(path).await
            }

            async fn read(&self, path: std::path::PathBuf) -> StorageResult<storage::Buffer> {
                self.get_storage()?.read(path).await
            }

            async fn write(&self, path: std::path::PathBuf, bs: Buffer) -> StorageResult<()> {
                self.get_storage()?.write(path, bs).await
            }

            async fn copy(&self, from: std::path::PathBuf, to: std::path::PathBuf) -> StorageResult<()> {
                self.get_storage()?.copy(from, to).await
            }

            async fn read_dir(&self, path: std::path::PathBuf) -> StorageResult<Vec<std::path::PathBuf>> {
                self.get_storage()?.read_dir(path).await
            }

            async fn remove_dir_all(&self, path: std::path::PathBuf) -> StorageResult<()> {
                self.get_storage()?.remove_dir_all(path).await
            }
        }
    };

    TokenStream::from(expanded)
}
