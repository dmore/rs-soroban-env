use super::RawVal;
use core::any;

pub trait EnvBase: Sized + Clone {
    // Used for recovering the concrete type of the Host.
    fn as_mut_any(&mut self) -> &mut dyn any::Any;

    // Used to check two environments are the same, trapping if not.
    fn check_same_env(&self, other: &Self);
}

///////////////////////////////////////////////////////////////////////////////
/// X-macro definition
///////////////////////////////////////////////////////////////////////////////

// The set of host functions need to be statically reflected-on in a variety of
// contexts (both in this crate and elsewhere in the guest and host crates), so
// we define them through an x-macro (a macro that calls a user-provided macro)
// and call the x-macro from all such contexts.
//
// How this macro works:
//  - It exports a higher-order "x-macro" called
//    call_macro_with_all_host_functions
//  - The x-macro takes the name of some callback macro to call
//  - The x-macro invokes the callback macro once, passing a single large token
//    tree, seen below in the body of the x-macro
//
// To use this macro:
//  - Call sites define a callback macro that matches on the token-tree
//  - Call sites invoke the x-macro passing their callback macro name
//
// The token-tree being passed is arbitrary, but is chosen to satisfy 3
// criteria:
//  - It's relatively easy to read, edit and understand its content
//  - It's easy to decompose with pattern-matching in the callback macros
//  - It contains everything any callback macro wants to match and use
//
// All callback macros have essentially the same token-tree matcher part,
// only their expansion parts differ.

#[macro_export]
macro_rules! call_macro_with_all_host_functions {

    // The x-macro takes a single ident, the name of a macro to call ...
    {$macro_to_call_back:ident} => {

        // ... and just calls it back, passing a single large token-tree.
        $macro_to_call_back! {

            // The token-tree we pass to the callback is a sequence of
            // blocks that have the following structure:
            //
            //  mod $mod_id:ident $mod_str:literal {
            //     ...
            //     { $fn_str:literal, fn $fn_id:ident $args:tt -> $ret:ty }
            //     ...
            //  }
            //
            // Where the sub token-tree $args:tt is a normal parenthesized
            // argument list of comma-separated arg:type pairs

            mod context "x" {
                {"$_", fn log_value(v:RawVal) -> RawVal }
                {"$0", fn get_last_operation_result() -> RawVal }
                {"$1", fn obj_cmp(a:RawVal, b:RawVal) -> i64 }
            }

            mod u64 "u" {
                {"$_", fn obj_from_u64(v:u64) -> RawVal }
                {"$0", fn obj_to_u64(v:RawVal) -> u64 }
            }

            mod i64 "i" {
                {"$_", fn obj_from_i64(v:i64) -> RawVal }
                {"$0", fn obj_to_i64(v:RawVal) -> i64 }
            }

            mod map "m" {
                {"$_", fn map_new() -> RawVal }
                {"$0", fn map_put(m:RawVal, k:RawVal, v:RawVal) -> RawVal}
                {"$1", fn map_get(m:RawVal, k:RawVal) -> RawVal}
                {"$2", fn map_del(m:RawVal, k:RawVal) -> RawVal}
                {"$3", fn map_len(m:RawVal) -> RawVal}
                {"$4", fn map_keys(m:RawVal) -> RawVal}
                {"$5", fn map_has(m:RawVal,k:RawVal) -> RawVal}
            }

            mod vec "v" {
                {"$_", fn vec_new() -> RawVal}
                {"$0", fn vec_put(v:RawVal, i:RawVal, x:RawVal) -> RawVal}
                {"$1", fn vec_get(v:RawVal, i:RawVal) -> RawVal}
                {"$2", fn vec_del(v:RawVal, i:RawVal) -> RawVal}
                {"$3", fn vec_len(v:RawVal) -> RawVal}

                {"$4", fn vec_push(v:RawVal, x:RawVal) -> RawVal}
                {"$5", fn vec_pop(v:RawVal) -> RawVal}
                {"$6", fn vec_take(v:RawVal, n:RawVal) -> RawVal}
                {"$7", fn vec_drop(v:RawVal, n:RawVal) -> RawVal}
                {"$8", fn vec_front(v:RawVal) -> RawVal}
                {"$9", fn vec_back(v:RawVal) -> RawVal}
                {"$A", fn vec_insert(v:RawVal, i:RawVal, x:RawVal) -> RawVal}
                {"$B", fn vec_append(v1:RawVal, v2:RawVal) -> RawVal}
            }

            mod ledger "l" {
                {"$_", fn get_current_ledger_num() -> RawVal }
                {"$0", fn get_current_ledger_close_time() -> RawVal}

                {"$1", fn pay(src:RawVal, dst:RawVal, asset:RawVal, amt:RawVal) -> RawVal}

                {"$2", fn put_contract_data(k:RawVal, v:RawVal) -> RawVal}
                {"$3", fn has_contract_data(k:RawVal) -> RawVal}
                {"$4", fn get_contract_data(k:RawVal) -> RawVal}
                {"$5", fn del_contract_data(k:RawVal) -> RawVal}

                {"$6", fn account_balance(acct:RawVal) -> RawVal}
                {"$7", fn account_trust_line(acct:RawVal, asset:RawVal) -> RawVal}
                {"$8", fn trust_line_balance(tl:RawVal) -> RawVal}
            }

            mod call "c" {
                {"$_", fn call0(contract:RawVal,func:RawVal) -> RawVal}
                {"$0", fn call1(contract:RawVal,func:RawVal,a:RawVal) -> RawVal}
                {"$1", fn call2(contract:RawVal,func:RawVal,a:RawVal,b:RawVal) -> RawVal}
                {"$2", fn call3(contract:RawVal,func:RawVal,a:RawVal,b:RawVal,c:RawVal) -> RawVal}
                {"$3", fn call4(contract:RawVal,func:RawVal,a:RawVal,b:RawVal,c:RawVal,d:RawVal) -> RawVal}
            }

            mod bigint "b" {
                {"$_", fn bigint_from_u64(x:RawVal) -> RawVal}
                {"$0", fn bigint_add(x:RawVal,y:RawVal) -> RawVal}
                {"$1", fn bigint_sub(x:RawVal,y:RawVal) -> RawVal}
                {"$2", fn bigint_mul(x:RawVal,y:RawVal) -> RawVal}
                {"$3", fn bigint_div(x:RawVal,y:RawVal) -> RawVal}
                {"$4", fn bigint_rem(x:RawVal,y:RawVal) -> RawVal}
                {"$5", fn bigint_and(x:RawVal,y:RawVal) -> RawVal}
                {"$6", fn bigint_or(x:RawVal,y:RawVal) -> RawVal}
                {"$7", fn bigint_xor(x:RawVal,y:RawVal) -> RawVal}
                {"$8", fn bigint_shl(x:RawVal,y:RawVal) -> RawVal}
                {"$9", fn bigint_shr(x:RawVal,y:RawVal) -> RawVal}
                {"$A", fn bigint_cmp(x:RawVal,y:RawVal) -> RawVal}
                {"$B", fn bigint_is_zero(x:RawVal) -> RawVal}
                {"$C", fn bigint_neg(x:RawVal) -> RawVal}
                {"$D", fn bigint_not(x:RawVal) -> RawVal}
                {"$E", fn bigint_gcd(x:RawVal) -> RawVal}
                {"$F", fn bigint_lcm(x:RawVal,y:RawVal) -> RawVal}
                {"$G", fn bigint_pow(x:RawVal,y:RawVal) -> RawVal}
                {"$H", fn bigint_pow_mod(p:RawVal,q:RawVal,m:RawVal) -> RawVal}
                {"$I", fn bigint_sqrt(x:RawVal) -> RawVal}
                {"$J", fn bigint_bits(x:RawVal) -> RawVal}
                {"$K", fn bigint_to_u64(x:RawVal) -> u64}
                {"$L", fn bigint_to_i64(x:RawVal) -> i64}
                {"$M", fn bigint_from_i64(x:i64) -> RawVal}
            }
        }
    };
}

///////////////////////////////////////////////////////////////////////////////
/// X-macro use: defining trait Env
///////////////////////////////////////////////////////////////////////////////

// This is a helper macro used only by generate_env_trait below. It consumes
// a token-tree of the form:
//
//  {fn $fn_id:ident $args:tt -> $ret:ty}
//
// and produces the the corresponding method declaration to be used in the Env
// trait.
macro_rules! host_function_helper {
    {fn $fn_id:ident($($arg:ident:$type:ty),*) -> $ret:ty}
    =>
    {
        fn $fn_id(&self, $($arg:$type),*) -> $ret;
    };
}

// This is a callback macro that pattern-matches the token-tree passed by the
// x-macro (call_macro_with_all_host_functions) and produces a suite of method
// declarations, which it places in the body of the declaration of the Env
// trait.
macro_rules! generate_env_trait {
    {
        $(
            // This outer pattern matches a single 'mod' block of the token-tree
            // passed from the x-macro to this macro. It is embedded in a `$()*`
            // pattern-repetition matcher so that it will match all provided
            // 'mod' blocks provided.
            mod $mod_id:ident $mod_str:literal
            {
                $(
                    // This inner pattern matches a single function description
                    // inside a 'mod' block in the token-tree passed from the
                    // x-macro to this macro. It is embedded in a `$()*`
                    // pattern-repetition matcher so that it will match all such
                    // descriptions.
                    { $fn_str:literal, fn $fn_id:ident $args:tt -> $ret:ty }
                )*
            }
        )*
    }

    => // The part of the macro above this line is a matcher; below is its expansion.

    {
        // This macro expands to a single item: the Env trait used to define the
        // interface implemented by Host and Guest, and used by client contract
        // code.
        pub trait Env: EnvBase
        {
            $(
                $(
                    // This invokes the host_function_helper! macro above
                    // passing only the relevant parts of the declaration
                    // matched by the inner pattern above. It is embedded in two
                    // nested `$()*` pattern-repetition expanders that
                    // correspond to the pattern-repetition matchers in the
                    // match section, but we ignore the structure of the 'mod'
                    // block repetition-level from the outer pattern in the
                    // expansion, flattening all functions from all 'mod' blocks
                    // into the Env trait.
                    host_function_helper!{fn $fn_id $args -> $ret}
                )*
            )*
        }
    };
}

// Here we invoke the x-macro passing generate_env_trait as its callback macro.
call_macro_with_all_host_functions! { generate_env_trait }