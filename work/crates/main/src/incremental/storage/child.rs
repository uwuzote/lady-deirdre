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

use crate::{
    arena::RefIndex,
    incremental::storage::{
        cache::{CacheEntry, ClusterCache},
        item::{Item, ItemRef, ItemRefVariant},
        page::Page,
    },
    lexis::{Length, TokenCount},
    std::*,
    syntax::Node,
};

pub(super) type ChildIndex = usize;
pub(super) type ChildCount = usize;

pub(crate) struct ChildRefIndex<N: Node> {
    pub(super) item: ItemRefVariant<N>,
    pub(super) index: ChildIndex,
}

impl<N: Node> Clone for ChildRefIndex<N> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<N: Node> Copy for ChildRefIndex<N> {}

impl<N: Node> ChildRefIndex<N> {
    #[inline(always)]
    pub(crate) const fn dangling() -> Self {
        Self {
            item: ItemRefVariant::dangling(),
            index: ChildIndex::MAX,
        }
    }

    #[inline(always)]
    pub(crate) const fn is_dangling(&self) -> bool {
        self.index == ChildIndex::MAX
    }

    // Safety:
    // 1. `self.item` and `other.item` are possibly dangling Page references.
    // 2. `self` and `other` belong to the same `Tree` instance.
    #[inline(always)]
    pub(crate) unsafe fn same_chunk_as(&self, other: &Self) -> bool {
        if self.index != other.index {
            return false;
        }

        if self.index != ChildIndex::MAX
            && unsafe { self.item.as_page_ref() != other.item.as_page_ref() }
        {
            return false;
        }

        true
    }

    // Safety:
    // 1. `self.item` and `other.item` are Page references.
    // 2. `self` and `other` belong to the same `Tree` instance.
    // 3. `self` is not ahead of `other`.
    // 4. `self` is not dangling.
    #[inline]
    pub(crate) unsafe fn continuous_to(&self, tail: &Self) -> Option<TokenCount> {
        debug_assert!(
            !self.is_dangling(),
            "Internal error. An attempt to access dangling ChildRefIndex.",
        );

        let head_page_ref = unsafe { self.item.as_page_ref() };
        let head_page = unsafe { head_page_ref.as_ref() };

        debug_assert!(
            self.index < head_page.occupied,
            "Internal error. ChildRefIndex index out of bounds.",
        );

        match tail.is_dangling() {
            false => {
                let tail_page_ref = unsafe { tail.item.as_page_ref() };

                match head_page_ref == tail_page_ref {
                    true => {
                        debug_assert!(
                            tail.index < head_page.occupied,
                            "Internal error. ChildRefIndex index out of bounds.",
                        );

                        match self.index <= tail.index {
                            true => Some(tail.index - self.index),

                            false => {
                                #[cfg(debug_assertions)]
                                {
                                    unreachable!("Internal error. Head is ahead of tail.");
                                }

                                #[allow(unreachable_code)]
                                unsafe {
                                    unreachable_unchecked()
                                }
                            }
                        }
                    }

                    false => {
                        if tail.index > 0 {
                            return None;
                        }

                        match &head_page.next {
                            Some(next) if next == tail_page_ref => {
                                Some(head_page.occupied - self.index)
                            }

                            _ => None,
                        }
                    }
                }
            }

            true => match head_page.next.is_some() {
                false => Some(head_page.occupied - self.index),

                true => None,
            },
        }
    }

    #[inline(always)]
    pub(super) fn make_dangle(&mut self) {
        self.index = ChildIndex::MAX;
    }

    // Safety:
    // 1. `self` is not dangling.
    // 2. `self.item` is a Page reference.
    // 3. `'a` does not outlive corresponding Page instance.
    // 4. There are no other mutable references to this span.
    #[inline(always)]
    pub(crate) unsafe fn span<'a>(&self) -> &'a Length {
        debug_assert!(
            !self.is_dangling(),
            "Internal error. An attempt to access dangling ChildRefIndex.",
        );

        let page = unsafe { self.item.as_page_ref().as_external_ref() };

        debug_assert!(
            self.index < page.occupied,
            "Internal error. ChildRefIndex index out of bounds.",
        );

        let span = unsafe { page.spans.get_unchecked(self.index) };

        debug_assert!(*span > 0, "Internal error. Zero span in Page.");

        span
    }

    // Safety:
    // 1. `self` is not dangling.
    // 2. `self.item` is a Page reference.
    // 3. `'a` does not outlive corresponding Page instance.
    // 4. There are no other mutable references to this String.
    #[inline(always)]
    pub(crate) unsafe fn string<'a>(&self) -> &'a str {
        debug_assert!(
            !self.is_dangling(),
            "Internal error. An attempt to access dangling ChildRefIndex.",
        );

        let page = unsafe { self.item.as_page_ref().as_external_ref() };

        debug_assert!(
            self.index < page.occupied,
            "Internal error. ChildRefIndex index out of bounds.",
        );

        let string = unsafe { page.strings.get_unchecked(self.index).assume_init_ref() };

        debug_assert!(!string.is_empty(), "Internal error. Empty string in Page.");

        string
    }

    // Safety:
    // 1. `self` is not dangling.
    // 2. `self.item` is a Page reference.
    // 3. `'a` does not outlive corresponding Page instance.
    // 4. There are no other mutable references to this Token.
    #[inline(always)]
    pub(crate) unsafe fn token<'a>(&self) -> &'a <N as Node>::Token {
        debug_assert!(
            !self.is_dangling(),
            "Internal error. An attempt to access dangling ChildRefIndex.",
        );

        let page = unsafe { self.item.as_page_ref().as_external_ref() };

        debug_assert!(
            self.index < page.occupied,
            "Internal error. ChildRefIndex index out of bounds.",
        );

        unsafe { page.tokens.get_unchecked(self.index).assume_init_ref() }
    }

    // Safety:
    // 1. `self` is not dangling.
    // 2. `self.item` is a Page reference.
    // 3. `'a` does not outlive corresponding Page instance.
    // 4. There are no other mutable references to this Token.
    #[inline(always)]
    pub(crate) unsafe fn token_mut<'a>(&self) -> &'a mut <N as Node>::Token {
        debug_assert!(
            !self.is_dangling(),
            "Internal error. An attempt to access dangling ChildRefIndex.",
        );

        let page = unsafe { self.item.as_page_ref().as_external_mut() };

        debug_assert!(
            self.index < page.occupied,
            "Internal error. ChildRefIndex index out of bounds.",
        );

        unsafe { page.tokens.get_unchecked_mut(self.index).assume_init_mut() }
    }

    // Safety:
    // 1. `self` is not dangling.
    // 2. `self.item` is a Page reference.
    // 3. `'a` does not outlive corresponding Page instance.
    // 4. There are no other mutable references to this ClusterCache.
    #[inline(always)]
    pub(crate) unsafe fn cache<'a>(&self) -> Option<&'a ClusterCache<N>> {
        debug_assert!(
            !self.is_dangling(),
            "Internal error. An attempt to access dangling ChildRefIndex.",
        );

        let page = unsafe { self.item.as_page_ref().as_external_ref() };

        debug_assert!(
            self.index < page.occupied,
            "Internal error. ChildRefIndex index out of bounds.",
        );

        match unsafe { page.clusters.get_unchecked(self.index).assume_init_ref() } {
            Some(cache_entry) => Some(&cache_entry.cache),

            None => None,
        }
    }

    // Safety:
    // 1. `self` is not dangling.
    // 2. `self.item` is a Page reference.
    // 3. `'a` does not outlive corresponding Page instance.
    // 4. There are no references to this ClusterCache.
    #[inline(always)]
    pub(crate) unsafe fn cache_mut<'a>(&self) -> Option<&'a mut ClusterCache<N>> {
        debug_assert!(
            !self.is_dangling(),
            "Internal error. An attempt to access dangling ChildRefIndex.",
        );

        let page = unsafe { self.item.as_page_ref().as_external_mut() };

        debug_assert!(
            self.index < page.occupied,
            "Internal error. ChildRefIndex index out of bounds.",
        );

        match unsafe {
            page.clusters
                .get_unchecked_mut(self.index)
                .assume_init_mut()
        } {
            Some(cache_entry) => Some(&mut cache_entry.cache),

            None => None,
        }
    }

    // Safety:
    // 1. `self` is not dangling.
    // 2. `self.item` is a Page reference.
    // 3. `'a` does not outlive corresponding Page instance.
    // 4. Referred item contains a cluster cache.
    #[inline(always)]
    pub(crate) unsafe fn cache_index(&self) -> RefIndex {
        debug_assert!(
            !self.is_dangling(),
            "Internal error. An attempt to access dangling ChildRefIndex.",
        );

        let page = unsafe { self.item.as_page_ref().as_external_ref() };

        debug_assert!(
            self.index < page.occupied,
            "Internal error. ChildRefIndex index out of bounds.",
        );

        match unsafe { page.clusters.get_unchecked(self.index).assume_init_ref() } {
            Some(cache_entry) => cache_entry.ref_index,

            None => {
                #[cfg(debug_assertions)]
                {
                    unreachable!(
                        "Internal error. An attempt to get RefIndex of undefined ClusterCache."
                    );
                }

                #[allow(unreachable_code)]
                unsafe {
                    unreachable_unchecked()
                }
            }
        }
    }

    // Safety:
    // 1. `self` is not dangling.
    // 2. `self.item` is a Page reference.
    // 3. Referred item contains a cluster cache.
    // 4. There are no other references to this ClusterCache.
    #[inline(always)]
    pub(crate) unsafe fn remove_cache(&self) -> RefIndex {
        debug_assert!(
            !self.is_dangling(),
            "Internal error. An attempt to access dangling ChildRefIndex.",
        );

        let page = unsafe { self.item.as_page_ref().as_external_mut() };

        debug_assert!(
            self.index < page.occupied,
            "Internal error. ChildRefIndex index out of bounds.",
        );

        match unsafe {
            take(
                page.clusters
                    .get_unchecked_mut(self.index)
                    .assume_init_mut(),
            )
        } {
            Some(cache_entry) => cache_entry.ref_index,

            None => {
                #[cfg(debug_assertions)]
                {
                    unreachable!("Internal error. An attempt to remove undefined ClusterCache.");
                }

                #[allow(unreachable_code)]
                unsafe {
                    unreachable_unchecked()
                }
            }
        }
    }

    // Safety:
    // 1. `self` is not dangling.
    // 2. `self.item` is a Page reference.
    // 3. If referred item contains valid CLusterCache, there are no external reference to that instance.
    #[inline(always)]
    pub(crate) unsafe fn set_cache(
        &self,
        ref_index: RefIndex,
        cache: ClusterCache<N>,
    ) -> Option<RefIndex> {
        debug_assert!(
            !self.is_dangling(),
            "Internal error. An attempt to access dangling ChildRefIndex.",
        );

        let page = unsafe { self.item.as_page_ref().as_external_mut() };

        debug_assert!(
            self.index < page.occupied,
            "Internal error. ChildRefIndex index out of bounds.",
        );

        let previous = replace(
            page.clusters
                .get_unchecked_mut(self.index)
                .assume_init_mut(),
            Some(CacheEntry { cache, ref_index }),
        );

        match previous {
            Some(cache_entry) => Some(cache_entry.ref_index),

            None => None,
        }
    }

    // Safety:
    // 1. `self` is not dangling.
    // 2. `self.item` is a Page reference.
    // 3. Referred item contains a cluster cache.
    // 4. There are no references to this ClusterCache.
    #[inline(always)]
    pub(crate) unsafe fn update_cache(&self, cache: ClusterCache<N>) {
        debug_assert!(
            !self.is_dangling(),
            "Internal error. An attempt to access dangling ChildRefIndex.",
        );

        let page = unsafe { self.item.as_page_ref().as_external_mut() };

        debug_assert!(
            self.index < page.occupied,
            "Internal error. ChildRefIndex index out of bounds.",
        );

        match unsafe {
            page.clusters
                .get_unchecked_mut(self.index)
                .assume_init_mut()
        } {
            Some(cache_entry) => {
                cache_entry.cache = cache;
            }

            None => {
                #[cfg(debug_assertions)]
                {
                    unreachable!("Internal error. An attempt to remove undefined ClusterCache.");
                }

                #[allow(unreachable_code)]
                unsafe {
                    unreachable_unchecked()
                }
            }
        }
    }

    // Safety:
    // 1. `self` is not dangling.
    // 2. `self.item` is a Page reference.
    #[inline(always)]
    pub(crate) unsafe fn chunk_ref_index(&self) -> RefIndex {
        debug_assert!(
            !self.is_dangling(),
            "Internal error. An attempt to access dangling ChildRefIndex.",
        );

        let page = unsafe { self.item.as_page_ref().as_external_ref() };

        debug_assert!(
            self.index < page.occupied,
            "Internal error. ChildRefIndex index out of bounds.",
        );

        unsafe { *page.chunks.get_unchecked(self.index) }
    }

    // Safety:
    // 1. `self` is not dangling.
    // 2. `self.item` is a Page reference.
    #[inline(always)]
    pub(crate) unsafe fn is_first(&self) -> bool {
        debug_assert!(
            !self.is_dangling(),
            "Internal error. An attempt to access dangling ChildRefIndex.",
        );

        let page = unsafe { self.item.as_page_ref().as_ref() };

        debug_assert!(
            self.index < page.occupied,
            "Internal error. ChildRefIndex index out of bounds.",
        );

        self.index == 0 && page.previous.is_none()
    }

    // Safety:
    // 1. `self` is not dangling.
    // 2. `self.item` is a Page reference.
    #[inline(always)]
    #[allow(unused)]
    pub(crate) unsafe fn is_last(&self) -> bool {
        debug_assert!(
            !self.is_dangling(),
            "Internal error. An attempt to access dangling ChildRefIndex.",
        );

        let page = unsafe { self.item.as_page_ref().as_ref() };

        debug_assert!(
            self.index < page.occupied,
            "Internal error. ChildRefIndex index out of bounds.",
        );

        self.index + 1 == page.occupied && page.next.is_none()
    }

    // Safety:
    // 1. `self` is not dangling.
    // 2. `self.item` is a Page reference.
    #[inline(always)]
    pub(crate) unsafe fn next(&mut self) {
        debug_assert!(
            !self.is_dangling(),
            "Internal error. An attempt to access dangling ChildRefIndex.",
        );

        let page = unsafe { self.item.as_page_ref().as_ref() };

        debug_assert!(
            self.index < page.occupied,
            "Internal error. ChildRefIndex index out of bounds.",
        );

        if self.index + 1 < page.occupied {
            self.index += 1;
            return;
        }

        match &page.next {
            None => {
                self.index = ChildIndex::MAX;
            }

            Some(next_ref) => {
                debug_assert!(
                    unsafe { next_ref.as_ref().occupied } >= Page::<N>::BRANCHING,
                    "Internal error. Incorrect Page balance."
                );

                self.item = unsafe { next_ref.into_variant() };
                self.index = 0;
            }
        }
    }

    // Safety:
    // 1. `self` is not dangling.
    // 2. `self.item` is a Page reference.
    #[inline(always)]
    pub(crate) unsafe fn back(&mut self) {
        debug_assert!(
            !self.is_dangling(),
            "Internal error. An attempt to access dangling ChildRefIndex.",
        );

        let page = unsafe { self.item.as_page_ref().as_ref() };

        debug_assert!(
            self.index < page.occupied,
            "Internal error. ChildRefIndex index out of bounds.",
        );

        if self.index > 0 {
            self.index -= 1;
            return;
        }

        match &page.previous {
            None => {
                self.index = ChildIndex::MAX;
            }

            Some(previous_ref) => {
                let previous_occupied = unsafe { previous_ref.as_ref().occupied };

                debug_assert!(
                    previous_occupied >= Page::<N>::BRANCHING,
                    "Internal error. Incorrect Page balance."
                );

                self.item = unsafe { previous_ref.into_variant() };
                self.index = previous_occupied - 1;
            }
        }
    }

    // Safety:
    // 1. `self` is not dangling.
    // 2. `self.item` is a Branch reference.
    #[inline(always)]
    pub(super) unsafe fn branch_span(&self) -> Length {
        debug_assert!(
            !self.is_dangling(),
            "Internal error. An attempt to get span from dangling ChildRefIndex.",
        );

        let branch = unsafe { self.item.as_branch_ref::<()>().as_ref() };

        debug_assert!(
            self.index < branch.inner.occupied,
            "Internal error. ChildRefIndex index is out of bounds.",
        );

        unsafe { *branch.inner.spans.get_unchecked(self.index) }
    }
}
