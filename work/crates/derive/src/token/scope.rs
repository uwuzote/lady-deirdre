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

use crate::utils::OptimizationStrategy;
use crate::{
    token::{characters::CharacterSet, terminal::Terminal, NULL},
    utils::{Automata, AutomataContext, Set, SetImpl, State},
};

pub(super) struct Scope {
    alphabet: CharacterSet,
    state: State,
    strategy: OptimizationStrategy,
}

impl AutomataContext for Scope {
    type Terminal = Terminal;

    #[inline(always)]
    fn gen_state(&mut self) -> State {
        let state = self.state;

        self.state += 1;

        state
    }

    #[inline(always)]
    fn strategy(&self) -> &OptimizationStrategy {
        &self.strategy
    }
}

impl Scope {
    #[inline(always)]
    pub(super) fn new(alphabet: CharacterSet) -> Self {
        Self {
            alphabet,
            state: 1,
            strategy: OptimizationStrategy::CANONICALIZE,
        }
    }

    #[inline(always)]
    pub(super) fn alphabet(&self) -> &CharacterSet {
        &self.alphabet
    }

    #[inline]
    pub(super) fn any(&mut self) -> Automata<Self> {
        let alphabet = self.alphabet.clone().into_inclusion(self);
        let other = self.other();

        self.union(alphabet, other)
    }

    #[inline]
    pub(super) fn other(&mut self) -> Automata<Self> {
        self.terminal(Set::new([Terminal::Character(NULL)]))
    }

    #[inline(always)]
    pub(super) fn reset(&mut self) {
        self.state = 1;
    }

    #[inline(always)]
    pub(super) fn set_strategy(&mut self, strategy: OptimizationStrategy) {
        self.strategy = strategy;
    }
}
