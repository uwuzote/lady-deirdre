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

use std::mem::take;

use proc_macro2::Ident;
use syn::{Error, Result};

use crate::node::{
    builder::Builder,
    regex::{operand::RegexOperand, span::SetSpan, Regex},
};

impl Inline for Regex {
    fn inline(&mut self, builder: &Builder) -> Result<()> {
        match self {
            Self::Operand(RegexOperand::Unresolved { name, capture }) => {
                match builder.get_inline(name) {
                    None => {
                        *self = Self::Operand(RegexOperand::Rule {
                            name: name.clone(),
                            capture: take(capture),
                        })
                    }

                    Some(inline) => {
                        let mut inline = inline.clone();

                        inline.set_span(name.span());

                        if let Some(target) = capture {
                            inline.capture(target)?;
                        }

                        *self = inline;
                    }
                };

                Ok(())
            }

            Self::Operand(RegexOperand::Debug { inner, .. }) => inner.inline(builder),

            Self::Operand(RegexOperand::Token { .. }) => Ok(()),

            Self::Operand(RegexOperand::Rule { .. }) => Ok(()),

            Self::Unary { inner, .. } => inner.inline(builder),

            Self::Binary { left, right, .. } => {
                left.inline(builder)?;
                right.inline(builder)?;

                Ok(())
            }
        }
    }

    fn capture(&mut self, target: &Ident) -> Result<()> {
        match self {
            Self::Operand(
                RegexOperand::Unresolved { capture, .. }
                | RegexOperand::Token { capture, .. }
                | RegexOperand::Rule { capture, .. },
            ) => {
                if let Some(capture) = capture {
                    if capture != target {
                        return Err(Error::new(
                            target.span(),
                            format!(
                                "Capturing variable \"{}\" conflicts with inner capturing variable \"{}\".",
                                target, capture
                            ),
                        ));
                    }
                }

                *capture = Some(target.clone());

                Ok(())
            }

            Self::Operand(RegexOperand::Debug { inner, .. }) => inner.capture(target),

            Self::Unary { inner, .. } => inner.capture(target),

            Self::Binary { left, right, .. } => {
                left.capture(target)?;
                right.capture(target)?;

                Ok(())
            }
        }
    }
}

pub(in crate::node) trait Inline {
    fn inline(&mut self, builder: &Builder) -> Result<()>;

    fn capture(&mut self, target: &Ident) -> Result<()>;
}
