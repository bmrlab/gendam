use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(StorageTrait)]
pub fn storage_trait_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // 检查结构体是否包含名为 `storage` 的字段
    let has_storage_field = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields_named) = &data_struct.fields {
            fields_named
                .named
                .iter()
                .any(|f: &syn::Field| f.ident.as_ref().map(|i| i == "storage").unwrap_or(false))
        } else {
            false
        }
    } else {
        false
    };

    // 检查结构体是否包含名为 `library` 的字段
    let has_library_field = if let Data::Struct(data_struct) = &input.data {
        if let Fields::Named(fields_named) = &data_struct.fields {
            fields_named
                .named
                .iter()
                .any(|f| f.ident.as_ref().map(|i| i == "library").unwrap_or(false))
        } else {
            false
        }
    } else {
        false
    };

    // 生成 `get_storage` 方法的实现
    let get_storage_impl = if has_storage_field {
        quote! {
            fn get_storage(&self) -> Storage {
                self.storage.clone()
            }
        }
    } else if has_library_field {
        quote! {
            fn get_storage(&self) -> Storage {
                self.library.storage.clone()
            }
        }
    } else {
        quote! {
            compile_error!("Struct must have either a `storage` field or a `library` field containing a `storage` field");
        }
    };

    let expanded = quote! {
        #[async_trait]
        impl StorageTrait for #name {
            #get_storage_impl

            fn get_actual_path(&self, path: std::path::PathBuf) -> std::path::PathBuf {
                self.get_storage().get_actual_path(path)
            }

            fn read_blocking(&self, path: std::path::PathBuf) -> StorageResult<storage::Buffer> {
                self.get_storage().read_blocking(path)
            }

            fn read_to_string(&self, path: std::path::PathBuf) -> StorageResult<String> {
                self.get_storage().read_to_string(path)
            }

            fn write_blocking(&self, path: std::path::PathBuf, bs: Bytes) -> StorageResult<()> {
                self.get_storage().write_blocking(path, bs)
            }

            fn remove_file(&self, path: std::path::PathBuf) -> StorageResult<()> {
                self.get_storage().remove_file(path)
            }

            async fn create_dir(&self, path: std::path::PathBuf) -> StorageResult<()> {
                self.get_storage().create_dir(path).await
            }

            async fn is_exist(&self, path: std::path::PathBuf) -> StorageResult<bool> {
                self.get_storage().is_exist(path).await
            }

            async fn read(&self, path: std::path::PathBuf) -> StorageResult<storage::Buffer> {
                self.get_storage().read(path).await
            }

            async fn write(&self, path: std::path::PathBuf, bs: Buffer) -> StorageResult<()> {
                self.get_storage().write(path, bs).await
            }

            async fn copy(&self, from: std::path::PathBuf, to: std::path::PathBuf) -> StorageResult<()> {
                self.get_storage().copy(from, to).await
            }

            async fn read_dir(&self, path: std::path::PathBuf) -> StorageResult<Vec<std::path::PathBuf>> {
                self.get_storage().read_dir(path).await
            }

            async fn remove_dir_all(&self, path: std::path::PathBuf) -> StorageResult<()> {
                self.get_storage().remove_dir_all(path).await
            }
        }
    };

    TokenStream::from(expanded)
}
