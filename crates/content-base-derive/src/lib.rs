extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(ContentTask)]
pub fn content_task_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let expanded = match input.data {
        Data::Enum(data_enum) => {
            let variant_matches_for_inner_run = data_enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                match &variant.fields {
                    Fields::Unnamed(fields) => {
                        let field_names: Vec<_> = fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, _)| {
                                syn::Ident::new(&format!("field{}", i), variant_name.span())
                            })
                            .collect();
                        quote! {
                            #name::#variant_name(#(#field_names),*) => {
                                #(#field_names.inner_run(file_info, ctx, task_run_record).await)*
                            }
                        }
                    }
                    Fields::Named(fields) => {
                        let field_names = fields.named.iter().map(|f| &f.ident);
                        let field_names = field_names.collect::<Vec<_>>();
                        quote! {
                            #name::#variant_name { #(#field_names),* } => {
                                #(#field_names.inner_run(file_info, ctx, task_run_record).await)*
                            }
                        }
                    }
                    Fields::Unit => quote! {
                        #name::#variant_name => Ok(())
                    },
                }
            });

            let variant_matches_for_task_parameters = data_enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                match &variant.fields {
                    Fields::Unnamed(fields) => {
                        let field_names: Vec<_> = fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, _)| {
                                syn::Ident::new(&format!("field{}", i), variant_name.span())
                            })
                            .collect();
                        quote! {
                            #name::#variant_name(#(#field_names),*) => {
                                #(#field_names.task_parameters(ctx))*
                            }
                        }
                    }
                    Fields::Named(fields) => {
                        let field_names = fields.named.iter().map(|f| &f.ident);
                        let field_names = field_names.collect::<Vec<_>>();
                        quote! {
                            #name::#variant_name { #(#field_names),* } => {
                                #(#field_names.task_parameters(ctx))*
                            }
                        }
                    }
                    Fields::Unit => quote! {
                        #name::#variant_name => json!({})
                    },
                }
            });

            let variant_matches_for_task_output = data_enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                match &variant.fields {
                    Fields::Unnamed(fields) => {
                        let field_names: Vec<_> = fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, _)| {
                                syn::Ident::new(&format!("field{}", i), variant_name.span())
                            })
                            .collect();
                        quote! {
                            #name::#variant_name(#(#field_names),*) => {
                                #(#field_names.task_output(task_run_record).await)*
                            }
                        }
                    }
                    Fields::Named(fields) => {
                        let field_names = fields.named.iter().map(|f| &f.ident);
                        let field_names = field_names.collect::<Vec<_>>();
                        quote! {
                            #name::#variant_name { #(#field_names),* } => {
                                #(#field_names.task_output(task_run_record).await)*
                            }
                        }
                    }
                    Fields::Unit => quote! {
                        #name::#variant_name => Ok(crate::record::TaskRunOutput::Data(json!({})))
                    },
                }
            });

            let variant_matches_for_task_deps = data_enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                match &variant.fields {
                    Fields::Unnamed(fields) => {
                        let field_names: Vec<_> = fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, _)| {
                                syn::Ident::new(&format!("field{}", i), variant_name.span())
                            })
                            .collect();
                        quote! {
                            #name::#variant_name(#(#field_names),*) => {
                                #(#field_names.task_dependencies())*
                            }
                        }
                    }
                    Fields::Named(fields) => {
                        let field_names = fields.named.iter().map(|f| &f.ident);
                        let field_names = field_names.collect::<Vec<_>>();
                        quote! {
                            #name::#variant_name { #(#field_names),* } => {
                                #(#field_names.task_dependencies())*
                            }
                        }
                    }
                    Fields::Unit => quote! {
                        #name::#variant_name => vec![] as Vec<crate::ContentTaskType>
                    },
                }
            });

            quote! {
                #[async_trait::async_trait]
                impl crate::ContentTask for #name {
                    async fn inner_run(&self, file_info: &crate::FileInfo, ctx: &content_base_context::ContentBaseCtx, task_run_record: &mut crate::record::TaskRunRecord) -> anyhow::Result<()> {
                        match self {
                            #(#variant_matches_for_inner_run)*
                        }
                    }

                    fn task_parameters(&self, ctx: &content_base_context::ContentBaseCtx) -> serde_json::Value {
                        match self {
                            #(#variant_matches_for_task_parameters)*
                        }
                    }

                    async fn task_output(&self, task_run_record: &crate::record::TaskRunRecord) -> anyhow::Result<crate::record::TaskRunOutput> {
                        match self {
                            #(#variant_matches_for_task_output)*
                        }
                    }

                    fn task_dependencies(&self) -> Vec<crate::ContentTaskType> {
                        match self {
                            #(#variant_matches_for_task_deps)*
                        }
                    }
                }
            }
        }
        _ => panic!("ContentTask can only be derived for enums"),
    };

    TokenStream::from(expanded)
}
