////////////////////////////////////////////////////////////////////////////////
// This file is a part of the "Lady Deirdre" Work,                            //
// a compiler front-end foundation technology.                                //
//                                                                            //
// This Work is a proprietary software with source available code.            //
//                                                                            //
// To copy, use, distribute, and contribute into this Work you must agree to  //
// the terms of the End User License Agreement:                               //
//                                                                            //
// https://github.com/Eliah-Lakhin/lady-deirdre/blob/master/EULA.md.          //
//                                                                            //
// The Agreement let you use this Work in commercial and non-commercial       //
// purposes. Commercial use of the Work is free of charge to start,           //
// but the Agreement obligates you to pay me royalties                        //
// under certain conditions.                                                  //
//                                                                            //
// If you want to contribute into the source code of this Work,               //
// the Agreement obligates you to assign me all exclusive rights to           //
// the Derivative Work or contribution made by you                            //
// (this includes GitHub forks and pull requests to my repository).           //
//                                                                            //
// The Agreement does not limit rights of the third party software developers //
// as long as the third party software uses public API of this Work only,     //
// and the third party software does not incorporate or distribute            //
// this Work directly.                                                        //
//                                                                            //
// AS FAR AS THE LAW ALLOWS, THIS SOFTWARE COMES AS IS, WITHOUT ANY WARRANTY  //
// OR CONDITION, AND I WILL NOT BE LIABLE TO ANYONE FOR ANY DAMAGES           //
// RELATED TO THIS SOFTWARE, UNDER ANY KIND OF LEGAL CLAIM.                   //
//                                                                            //
// If you do not or cannot agree to the terms of this Agreement,              //
// do not use this Work.                                                      //
//                                                                            //
// Copyright (c) 2022 Ilya Lakhin (Илья Александрович Лахин).                 //
// All rights reserved.                                                       //
////////////////////////////////////////////////////////////////////////////////

use proc_macro2::{Ident, TokenStream};

use crate::{
    token::variant::TokenVariant,
    utils::{debug_panic, Facade},
};

pub(super) type RuleIndex = usize;
pub(super) type RulePrecedence = usize;

pub(super) struct RuleMeta {
    name: Ident,
    index: RuleIndex,
    derive_in_use: bool,
    constructor: Option<Ident>,
}

impl From<TokenVariant> for RuleMeta {
    #[inline]
    fn from(variant: TokenVariant) -> Self {
        match variant {
            TokenVariant::Rule {
                name,
                index,
                constructor,
                ..
            } => Self {
                name,
                index,
                derive_in_use: false,
                constructor,
            },

            _ => debug_panic!("Non-rule variant."),
        }
    }
}

impl RuleMeta {
    #[inline]
    pub(super) fn public_index(&self) -> RuleIndex {
        self.index + 1
    }

    #[inline]
    pub(super) fn uses_token_variable(&self) -> bool {
        self.derive_in_use && self.constructor.is_none()
    }

    #[inline]
    pub(super) fn uses_kind_variable(&self) -> bool {
        self.derive_in_use && self.constructor.is_some()
    }

    #[inline]
    pub(super) fn output_in_place(&self, facade: &Facade) -> TokenStream {
        match &self.constructor {
            None => {
                let name = &self.name;

                quote! {
                    Self::#name
                }
            }

            Some(constructor) => {
                let core = facade.core_crate();

                let span = constructor.span();

                quote_spanned! {span=>
                    Self::#constructor(#core::lexis::LexisSession::substring(session))
                }
            }
        }
    }

    #[inline]
    pub(super) fn output_derive(&mut self) -> TokenStream {
        self.derive_in_use = true;

        match &self.constructor {
            None => {
                let name = &self.name;

                quote! {
                    token = Self::#name
                }
            }

            Some(..) => {
                let index = self.public_index();

                quote! {
                    kind = #index
                }
            }
        }
    }
}
