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

use crate::{lexis::Site, std::*};

pub(crate) const NULL: char = '\0';

// Safety: `characters` is not empty.
#[inline(always)]
pub(crate) unsafe fn get_lexis_character(mut characters: Chars<'_>) -> char {
    match characters.next() {
        Some(character) => {
            if character == NULL {
                return char::REPLACEMENT_CHARACTER;
            }

            character
        }

        None => {
            #[cfg(debug_assertions)]
            {
                unreachable!("Internal error. Empty characters iterator.");
            }

            #[allow(unreachable_code)]
            unsafe {
                unreachable_unchecked()
            }
        }
    }
}

#[inline]
pub(crate) fn split_left(string: &str, mut site: Site) -> &str {
    if site == 0 {
        return "";
    }

    for (index, _) in string.char_indices() {
        if site == 0 {
            return unsafe { string.get_unchecked(0..index) };
        }

        site -= 1;
    }

    string
}

#[inline]
pub(crate) fn split_right(string: &str, mut site: Site) -> &str {
    if site == 0 {
        return string;
    }

    for (index, _) in string.char_indices() {
        if site == 0 {
            return unsafe { string.get_unchecked(index..string.len()) };
        }

        site -= 1;
    }

    ""
}
