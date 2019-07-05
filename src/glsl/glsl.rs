
//TODO add default layout thing for shader storage objects and uniform buffer objects
//TODO capture the first element name of unnamed structs in order to fix their indexing

#[macro_export]
macro_rules! glsl {

    //
    //--------------------------------------------------------------------------------------------
    //helpful subroutines and functions that do not rely on parser memory storage
    //--------------------------------------------------------------------------------------------
    //

    //
    //A subroutine for getting the rust version of every possible glsl type
    //This includes the base glsl primitives as well dlsl defined structs and arrays
    //

    //primitives have already been aliased as their glsl names so we can just copy paste the name.
    //the only exception is bool since that type name is a rust primitive as well
    ([bool] @ty $($code:tt)*) => {glsl!([c_bool] $($code)*);};

    //this captures all constructed types and primitives (besides bool)
    ([$name:ident] @ty $($code:tt)*) => {glsl!([$name] $($code)*);};

    //for capturing the array types
    ([$name:ident $([$($s:tt)*])*] @ty $($code:tt)*) => {glsl!([$name] @ty $([$($s)*])* @ty $($code)*);};
    ([{$($fields:tt)*} $([$($s:tt)*])+] @ty $($code:tt)*) => {glsl!([{$($fields)*}] @ty $([$($s)*])+ @ty $($code)*);};

    //for arrays, we need to reverse the order of the brackets since glsl has inner on the right
    ([$($ty:tt)*] [] @ty $($code:tt)*) => {glsl!([$($ty)*] [] @arr $($code)*);};
    ([$($ty:tt)*] [] $([$($s:tt)*])+ @ty $($code:tt)*) => {glsl!([$($ty)*] $([$($s)*])* @ty [] @arr $($code)*);};
    ([$($ty:tt)*] [$len:expr] @ty $($code:tt)*) => {glsl!([$($ty)*] [$len] @arr $($code)*);};
    ([$($ty:tt)*] [$len:expr] $([$($s:tt)*])+ @ty $($code:tt)*) => {glsl!([$($ty)*] $([$($s)*])+ @ty [$len] @arr $($code)*);};

    //next, nest the arrays
    ([$($ty:tt)*] @arr $($code:tt)*) => {glsl!( [$($ty)*] $($code)*);};
    ([$($ty:tt)*] [] @arr $($code:tt)*) => {glsl!([[$($ty)*]] $($code)*);};
    ([$($ty:tt)*] [$len:expr] @arr $($code:tt)*) => {glsl!([[$($ty)*; $len]] $($code)*);};

    //for interface blocks, we want to put everything into a tuple if it is more than one field,
    //or just pass through the single field if only one.
    ([{$($fields:tt)*}] @ty $($code:tt)*) => {glsl!({$($fields)*} @decl_list @ty $($code)*);};
    ([($($fields:tt)*)] @ty $($code:tt)*) => {glsl!({$($fields)*} @param_list @ty $($code)*);};
    ({$([$name:ident: $($ty:tt)*])*} @ty $($code:tt)*) => {glsl!([($($($ty)*),*)] $($code)*);};

    //if there's something that doesn't parse, throw a compiler error
    ([$($ty:tt)*] @ty $($code:tt)*) => {compile_error!(concat!("Unrecognized GLSL type: ", stringify!($($ty)*)));};

    //
    //A subroutine for parameter lists
    //

    ({$($params:tt)*} @param_list $($code:tt)*) => {glsl!({} {$($params)*} @param_list $($code)*);};

    //disregard all of the mem access keywords
    ({$($rust:tt)*} {in $($params:tt)*} @param_list $($code:tt)*) => { glsl!({$($rust)*} {$($params)*} @param_list $($code)*);};
    ({$($rust:tt)*} {out $($params:tt)*} @param_list $($code:tt)*) => { glsl!({$($rust)*} {$($params)*} @param_list $($code)*);};
    ({$($rust:tt)*} {inout $($params:tt)*} @param_list $($code:tt)*) => { glsl!({$($rust)*} {$($params)*} @param_list $($code)*);};

    //parse out the list of types from a single declaration
    ({$($rust:tt)*} {} @param_list $($code:tt)*) => { glsl!({$($rust)*} $($code)*); };
    ({$($rust:tt)*} {$type:ident $name:ident $([$($s:tt)*])*} @param_list $($code:tt)*) => {
        glsl!([$type $([$($s)*])*] @ty {$($rust)*} $name {} @param_list $($code)*);
    };
    ({$($rust:tt)*} {$type:ident $name:ident $([$($s:tt)*])*, $($rest:tt)*} @param_list $($code:tt)*) => {
        glsl!([$type $([$($s)*])*] @ty {$($rust)*} $name {$($rest)*} @param_list $($code)*);
    };
    ([$($ty:tt)*] {$($rust:tt)*} $name:ident {$($rest:tt)*} @param_list $($code:tt)*) => {
        glsl!({$($rust)* [$name: $($ty)*]} {$($rest)*} @param_list $($code)*);
    };

    //
    //A subroutine for parsing a declaration list
    //

    ({$($fields:tt)*} @decl_list $($code:tt)*) => {glsl!({} {$($fields)*} @decl_list $($code)*);};

    //disregard all of the mem access keywords
    ({$($rust:tt)*} {coherent $($fields:tt)*} @decl_list $($code:tt)*) => { glsl!({$($rust)*} {$($fields)*} @decl_list $($code)*);};
    ({$($rust:tt)*} {volatile $($fields:tt)*} @decl_list $($code:tt)*) => { glsl!({$($rust)*} {$($fields)*} @decl_list $($code)*);};
    ({$($rust:tt)*} {restrict $($fields:tt)*} @decl_list $($code:tt)*) => { glsl!({$($rust)*} {$($fields)*} @decl_list $($code)*);};
    ({$($rust:tt)*} {readonly $($fields:tt)*} @decl_list $($code:tt)*) => { glsl!({$($rust)*} {$($fields)*} @decl_list $($code)*);};
    ({$($rust:tt)*} {writeonly $($fields:tt)*} @decl_list $($code:tt)*) => { glsl!({$($rust)*} {$($fields)*} @decl_list $($code)*);};

    //parse out the list of types from a single declaration
    ({$($rust:tt)*} {} @decl_list $($code:tt)*) => { glsl!({$($rust)*} $($code)*); };
    ({$($rust:tt)*} {$type:ident $($names:ident $([$($s:tt)*])*),*; $($fields:tt)*} @decl_list $($code:tt)*) => {
        glsl!({$($rust)*} [$type $($names $([$($s)*])*,)*] @decl_list {$($fields)*} @decl_list $($code)*);
    };

    //split up a single line of variable declarations
    ([$($ty:tt)*] {$($rust:tt)*} $name:ident [$($rest:tt)*] @decl_list $($code:tt)*) => {
        glsl!({$($rust)* [$name: $($ty)*]} [$($rest)*] @decl_list $($code)*);
    };
    ({$($rust:tt)*} [$ty:ident] @decl_list $($code:tt)*) => {glsl!({$($rust)*} $($code)*);};
    ({$($rust:tt)*} [$ty:ident $name:ident $([$($s:tt)*])*, $($rest:tt)*] @decl_list $($code:tt)*) => {
        glsl!( [$ty $([$($s)*])*] @ty {$($rust)*} $name [$ty $($rest)*] @decl_list $($code)*);
    };


    //
    //A subroutine for getting unique generic identifiers
    //

    //for attributes
    ([T0] @next_generic $($code:tt)*) => { glsl!([T1] $($code)*); };
    ([T1] @next_generic $($code:tt)*) => { glsl!([T2] $($code)*); };
    ([T2] @next_generic $($code:tt)*) => { glsl!([T3] $($code)*); };
    ([T3] @next_generic $($code:tt)*) => { glsl!([T4] $($code)*); };
    ([T4] @next_generic $($code:tt)*) => { glsl!([T5] $($code)*); };
    ([T5] @next_generic $($code:tt)*) => { glsl!([T6] $($code)*); };
    ([T6] @next_generic $($code:tt)*) => { glsl!([T7] $($code)*); };
    ([T7] @next_generic $($code:tt)*) => { glsl!([T8] $($code)*); };
    ([T8] @next_generic $($code:tt)*) => { glsl!([T9] $($code)*); };
    ([T9] @next_generic $($code:tt)*) => { glsl!([Ta] $($code)*); };
    ([Ta] @next_generic $($code:tt)*) => { glsl!([Tb] $($code)*); };
    ([Tb] @next_generic $($code:tt)*) => { glsl!([Tc] $($code)*); };
    ([Tc] @next_generic $($code:tt)*) => { glsl!([Td] $($code)*); };
    ([Td] @next_generic $($code:tt)*) => { glsl!([Te] $($code)*); };
    ([Te] @next_generic $($code:tt)*) => { glsl!([Tf] $($code)*); };
    ([Tf] @next_generic $($code:tt)*) => { compile_error!("Too many vertex attributes! (Max=16)") };

    //for the access parameter for the buffers for interface blocks
    ([A0] @next_generic $($code:tt)*) => { glsl!([A1] $($code)*); };
    ([A1] @next_generic $($code:tt)*) => { glsl!([A2] $($code)*); };
    ([A2] @next_generic $($code:tt)*) => { glsl!([A3] $($code)*); };
    ([A3] @next_generic $($code:tt)*) => { glsl!([A4] $($code)*); };
    ([A4] @next_generic $($code:tt)*) => { glsl!([A5] $($code)*); };
    ([A5] @next_generic $($code:tt)*) => { glsl!([A6] $($code)*); };
    ([A6] @next_generic $($code:tt)*) => { glsl!([A7] $($code)*); };
    ([A7] @next_generic $($code:tt)*) => { glsl!([A8] $($code)*); };
    ([A8] @next_generic $($code:tt)*) => { glsl!([A9] $($code)*); };
    ([A9] @next_generic $($code:tt)*) => { glsl!([Aa] $($code)*); };
    ([Aa] @next_generic $($code:tt)*) => { glsl!([Ab] $($code)*); };
    ([Ab] @next_generic $($code:tt)*) => { glsl!([Ac] $($code)*); };
    ([Ac] @next_generic $($code:tt)*) => { glsl!([Ad] $($code)*); };
    ([Ad] @next_generic $($code:tt)*) => { glsl!([Ae] $($code)*); };
    ([Ae] @next_generic $($code:tt)*) => { glsl!([Af] $($code)*); };
    ([Af] @next_generic $($code:tt)*) => { compile_error!("Too many interface blocks! (Max=16)") };

    //increment the generics for every entry in the bucket
    ([$T:ident] {[$($a:tt)*] $($rest:tt)*} @generic $($code:tt)*) => {glsl!([$T] @next_generic {$($rest)*} @generic $($code)*);};
    ([$T:ident] {} @generic $($code:tt)*) => {glsl!([$T] $($code)*);};

    //
    //Subroutine for stringifying glsl code without all the extra newlines
    //

    (@stringify ) => { "" };
    (@stringify {$($lines:tt)*} $($rest:tt)*) => { concat!("{\n", glsl!(@stringify $($lines)*) ,"}", glsl!(@stringify $($rest)*)) };
    (@stringify ($($c:tt)*) $($rest:tt)*) => { concat!("(", glsl!(@stringify $($c)*) ,")", glsl!(@stringify $($rest)*)) };
    (@stringify [$($c:tt)*] $($rest:tt)*) => { concat!("[", glsl!(@stringify $($c)*) ,"]", glsl!(@stringify $($rest)*)) };
    (@stringify $c:ident $($rest:tt)*) => { concat!(stringify!($c), " ", glsl!(@stringify $($rest)*)) };
    (@stringify ; $($rest:tt)*) => { concat!(";\n", glsl!(@stringify $($rest)*)) };
    (@stringify $c:tt $($rest:tt)*) => { concat!(stringify!($c), glsl!(@stringify $($rest)*)) };

    //
    //--------------------------------------------------------------------------------------------
    //Control Structures
    //--------------------------------------------------------------------------------------------
    //

    //for easy branching
    ([true] @if {$($c1:tt)*} @then {$($c2:tt)*} @else $($code:tt)*) => {glsl!($($c1)* $($code)*);};
    ([false] @if {$($c1:tt)*} @then {$($c2:tt)*} @else $($code:tt)*) => {glsl!($($c2)* $($code)*);};
    (($($data:tt)*) [true] @if {$($c1:tt)*} @then {$($c2:tt)*} @else $($code:tt)*) => {glsl!(($($data)*) $($c1)* $($code)*);};
    (($($data:tt)*) [false] @if {$($c1:tt)*} @then {$($c2:tt)*} @else $($code:tt)*) => {glsl!(($($data)*) $($c2)* $($code)*);};

    //for rearranging arguments and tokens
    (@lbl $i:tt @swap $($code:tt)*) => {glsl!($i $($code)*);};
    (@lbl $i:tt $j:tt @swap $($code:tt)*) => {glsl!($i $j $($code)*);};
    ($i:tt @lbl $j:tt @swap $($code:tt)*) => {glsl!($j $i $($code)*);};
    (@lbl $i:tt $j:tt $k:tt @swap $($code:tt)*) => {glsl!($i $j $k $($code)*);};
    ($i:tt @lbl $j:tt $k:tt @swap $($code:tt)*) => {glsl!($j $k $i $($code)*);};
    ($i:tt $j:tt @lbl $k:tt @swap $($code:tt)*) => {glsl!($k $i $j $($code)*);};
    (@lbl $i:tt $j:tt $k:tt $l:tt @swap $($code:tt)*) => {glsl!($i $j $k $l $($code)*);};
    ($i:tt @lbl $j:tt $k:tt $l:tt @swap $($code:tt)*) => {glsl!($j $k $l $i $($code)*);};
    ($i:tt $j:tt @lbl $k:tt $l:tt @swap $($code:tt)*) => {glsl!($k $l $i $j $($code)*);};
    ($i:tt $j:tt $k:tt @lbl $l:tt @swap $($code:tt)*) => {glsl!($l $i $j $k $($code)*);};

    //for compiling together arguments in a particular order
    ($i:tt {$($expr:tt)*} @eval $($code:tt)*) => {glsl!($($expr)* @lbl $i @swap $($code)*);};
    ($i:tt $j:tt {$($expr:tt)*} @eval $($code:tt)*) => {glsl!($($expr)* @lbl $i $j @swap $($code)*);};
    ($i:tt $j:tt $k:tt {$($expr:tt)*} @eval $($code:tt)*) => {glsl!($($expr)* @lbl $i $j $k @swap $($code)*);};

    //for quoting code
    ($data:tt [$($code:tt)*] @quote $($tail:tt)*) => {$($code)*};
    ([$($code:tt)*] @quote $($tail:tt)*) => {$($code)*};

    //for calling other macros
    ($fun:ident @call $($tail:tt)*) => {$fun!(); glsl!($($tail)*);};
    ($i:tt $fun:ident @call $($tail:tt)*) => {$fun!($i); glsl!($($tail)*);};
    ($i:tt $j:tt $fun:ident @call $($tail:tt)*) => {$fun!($i $j); glsl!($($tail)*);};
    ($i:tt $j:tt $k:tt $fun:ident @call $($tail:tt)*) => {$fun!($i $j $k); glsl!($($tail)*);};
    ($i:tt $j:tt $k:tt $l:tt $fun:ident @call $($tail:tt)*) => {$fun!($i $j $k $l); glsl!($($tail)*);};

    //some simple arithmetic
    ([$($t:tt)*] @count $($code:tt)*) => {glsl!([$($t)*] [0] @count $($code)*);};
    ([] [$val:expr] @count $($code:tt)*) => {glsl!([$val] $($code)*);};
    ([$t0:tt $($t:tt)*] [$val:expr] @count $($code:tt)*) => {glsl!([$($t)*] [$val + 1] @count $($code)*);};

    //logical operation
    ([false] [$b:tt] @and $($code:tt)*) => {glsl!([false] $($code)*);};
    ([$b:tt] [false] @and $($code:tt)*) => {glsl!([false] $($code)*);};
    ([true]  [true]  @and $($code:tt)*) => {glsl!([true]  $($code)*);};

    ([true]  [$b:tt] @or  $($code:tt)*) => {glsl!([true]  $($code)*);};
    ([$b:tt] [true]  @or  $($code:tt)*) => {glsl!([true]  $($code)*);};
    ([false] [false] @or  $($code:tt)*) => {glsl!([false] $($code)*);};

    ([false] [false] @xor $($code:tt)*) => {glsl!([false] $($code)*);};
    ([true]  [false] @xor $($code:tt)*) => {glsl!([true]  $($code)*);};
    ([false] [true]  @xor $($code:tt)*) => {glsl!([true]  $($code)*);};
    ([true]  [true]  @xor $($code:tt)*) => {glsl!([false] $($code)*);};

    (@ret) => {};

    //for getting rid of unnecessary parameters
    ($i:tt @ignore $($code:tt)*) => {glsl!($($code)*);};
    ($i:tt $j:tt @ignore $($code:tt)*) => {glsl!($i $($code)*);};
    ($i:tt $j:tt $k:tt @ignore $($code:tt)*) => {glsl!($i $j $($code)*);};
    ($i:tt $j:tt $k:tt $l:tt @ignore $($code:tt)*) => {glsl!($i $j $k $($code)*);};

    //
    //--------------------------------------------------------------------------------------------
    //Primary Parser data management and controls
    //--------------------------------------------------------------------------------------------
    //

    //
    //Basic bucket management
    //(note that this works no matter how many or what names the buckets are
    //so long as we update the macro used for name checking)
    //

    //for matching each bucket name
    (uni         uni         @bucket $($code:tt)*) => {glsl!([true]  $($code)*);};
    (attr        attr        @bucket $($code:tt)*) => {glsl!([true]  $($code)*);};
    (block       block       @bucket $($code:tt)*) => {glsl!([true]  $($code)*);};
    (src         src         @bucket $($code:tt)*) => {glsl!([true]  $($code)*);};
    (flags       flags       @bucket $($code:tt)*) => {glsl!([true]  $($code)*);};
    (fun         fun         @bucket $($code:tt)*) => {glsl!([true]  $($code)*);};
    (current     current     @bucket $($code:tt)*) => {glsl!([true]  $($code)*);};
    ($name:ident $test:ident @bucket $($code:tt)*) => {glsl!([false] $($code)*);};

    //recursively loop until we find the bucket
    (($d:tt @$test:ident $bucket:tt $($rest:tt)*) $name:ident @bucket $($code:tt)*) => {
        glsl!(
            $name $test @bucket @if {
                ($d $($rest)*) $name $bucket
            } @then {
                ($d $($rest)* @$test $bucket) $name @bucket
            } @else $($code)*
        );
    };

    //put a bucket back up front
    (($d:tt $($data:tt)*) $name:ident $bucket:tt @push_bucket $($code:tt)*) => {
        glsl!(($d @$name $bucket $($data)*) $($code)*);
    };

    //for directly overwriting what's in a bucket
    ($data:tt $name:ident $new:tt @set_bucket $($code:tt)*) => {
        glsl!($data $name @bucket $new @set_bucket $($code)*);
    };
    ($data:tt $name:ident $old:tt $new:tt @set_bucket $($code:tt)*) => {
        glsl!($data $name $new @push_bucket $($code)*);
    };

    //
    //Adding entries to a bucket
    //

    ([$($new:tt)*] $data:tt $bucket:ident @add_to $($code:tt)*) => { glsl!($data [$($new)*] $bucket @add_to $($code)*); };
    ($data:tt [$($new:tt)*] $bucket:ident @add_to $($code:tt)*) => {
        glsl!($data $bucket @bucket [$($new)*] @add_to $($code)*);
    };

    (($d:tt $($data:tt)*) $bucket:ident {$($cont:tt)*} [$name:ident $($new:tt)*] @add_to $($code:tt)*) => {
        glsl!(
            $name {$($cont)*} @contains_name @if {
                ($d $($data)*) $bucket {$($cont)*} @push_bucket
                [false]
            } @then {
                $d $name _register_name @call
                ($d $($data)*) $bucket {$($cont)* [$name $($new)*]} @push_bucket
                [true]
            } @else $($code)*);
    };

    //
    //Checking if a name is already registered
    //

    ($name:ident {} @contains_name $($code:tt)*) => { glsl!([false] $($code)*); };
    ($name:ident {[$test:ident $($data:tt)*] $($rest:tt)*} @contains_name $($code:tt)*) => {
        $test!( $name glsl @if {[true]} @then {$name {$($rest)*} @contains_name} @else $($code)* );
    };


    //
    //Storing uniform and attribute data
    //

    ([$ty:ty] $name:ident @format_field $($code:tt)*) => {glsl!([$name: $ty] $($code)*);};
    ($data:tt [$($ty:tt)*] $name:ident @attribute $($code:tt)*) => {
        glsl!([$($ty)*] @ty $name @format_field $data attr @add_to @ignore $($code)*);
    };
    ($data:tt [$($ty:tt)*] $name:ident @uniform $($code:tt)*) => {
        glsl!([$($ty)*] @ty $name @format_field $data uni @add_to @ignore $($code)*);
    };

    //
    //Uniform and Shader Storage Blocks
    //

    //parse through and find the mem layout

    //the supported layouts
    ($data:tt layout(std140 $($rest:tt)*) @block_layout $($code:tt)*) => {glsl!($data [std140] $($code)*);};
    ($data:tt layout(std430 $($rest:tt)*) @block_layout $($code:tt)*) => {glsl!($data [std430] $($code)*);};
    ($data:tt layout(shared $($rest:tt)*) @block_layout $($code:tt)*) => {glsl!($data [shared] $($code)*);};
    ($data:tt layout(packed $($rest:tt)*) @block_layout $($code:tt)*) => {compile_error!("Packed layout currently not supported");};

    //get rid of all irrelevant params
    ($data:tt layout($p:ident = $val:expr, $($rest:tt)*) @block_layout $($code:tt)*) => {glsl!($data layout($($rest)*) @block_layout $($code)*);};
    ($data:tt layout($p:ident, $($rest:tt)*) @block_layout $($code:tt)*) => {glsl!($data layout($($rest)*) @block_layout $($code)*);};

    //defaults to shared
    ($data:tt layout() @block_layout $($code:tt)*) => {glsl!($data [shared] $($code)*);};
    ($data:tt layout($p:ident = $val:expr) @block_layout $($code:tt)*) => {glsl!($data [shared] $($code)*);};
    ($data:tt layout($p:ident) @block_layout $($code:tt)*) => {glsl!($data [shared] $($code)*);};


    //get the bucket block and sub in the proper block type
    ($data:tt [$layout:ident] [$($ty:tt)*] $name:ident uniform @block $($code:tt)*) => {
        glsl!($data block @bucket [$layout] [$($ty)*] $name UniformBlock @block $($code)*);
    };
    ($data:tt [$layout:ident] [$($ty:tt)*] $name:ident buffer @block $($code:tt)*) => {
        glsl!($data block @bucket [$layout] [$($ty)*] $name ShaderStorageBlock @block $($code)*);
    };

    //get all of the data regarding the block
    (($d:tt $($data:tt)*) block {$($blocks:tt)*} [$layout:ident] [$($ty:tt)*] $name:ident $kind:ident @block $($code:tt)*) => {
        glsl! {

            $name {$($blocks)*} @contains_name

            @if {
                ($d @block {$($blocks)*} $($data)*)
            } @then {
                //get a unique generic
                [A0] {$($blocks)*} @generic

                //get the Rust type from the glsl type
                { [$($ty)*] @ty } @eval

                //return to glsl
                ($d $($data)*) {$($blocks)*} [$layout] $name $kind @block
            } @else $($code)*
        }
    };

    //register a uniform block
    ([$A:ident] [$ty:ty] ($d:tt $($data:tt)*) {$($blocks:tt)*} [$layout:ident] $name:ident $kind:ident @block $($code:tt)*) => {
        _register_name!($d $name);
        glsl!(($d @block {$($blocks)* [$name<$kind, $layout, $A>: $ty]} $($data)*) $($code)*);
    };

    //
    //Register a shader and store it's source code
    //

    //store the shader's source code
    ($data:tt @src $($code:tt)* ) => { glsl!($data current @bucket @src $($code)* ); };
    ($data:tt current {Lib {$first:expr; $last:expr}} @src $($code:tt)* ) => { glsl!($data current {} @push_bucket $($code)* ); };
    ($data:tt current {$shdr:ident {$first:expr; $last:expr}} @src $($code:tt)* ) => {
        glsl!($data current {} @push_bucket src @bucket $shdr {($first+$last).as_str()} @src $($code)* );
    };
    ($data:tt src {$($shaders:tt)*} $name:ident {$shader:expr} @src $($code:tt)* ) => {
        glsl!($data src {$($shaders)* [$name=$shader]} @push_bucket $($code)* );
    };

    //
    //Set a flag
    //

    //get the flag bucket
    ($data:tt [$val:tt] [$($id:tt)*] @set_flag $($code:tt)* ) => {
        glsl!($data flags @bucket {} [$val] [$($id)*] @set_flag $($code)* );
    };

    //find the flag by counting down in unary
    ($data:tt flags {[$v:tt] $($f:tt)*} {$($f2:tt)*} [$val:tt] [$i:tt $($id:tt)*] @set_flag $($code:tt)* ) => {
        glsl!($data flags {$($f)*} {$($f2)* [$v]} [$val] [$($id)*] @set_flag $($code)* );
    };
    (($d:tt $($data:tt)*) flags {[$v:tt] $($f:tt)*} {$($f2:tt)*} [$val:tt] [] @set_flag $($code:tt)* ) => {
        glsl!(($d @flags {$($f2)* [$val] $($f)*} $($data)*) $($code)* );
    };

    //
    //for managing the current shader's parsing info
    //

    //start a new set of parsing data
    ($data:tt $shdr:ident @new_shdr $($code:tt)*) => {
        glsl!($data current {$shdr {"".to_owned(); ""}} @set_bucket $($code)*);
    };

    //check if the current shader is a vertex shader
    ($data:tt @is_vertex $($code:tt)*) => { glsl!($data current @bucket @is_vertex $($code)*); };
    ($data:tt current {Vertex $src:tt} @is_vertex $($code:tt)*) => {
        glsl!($data current {Vertex $src} @push_bucket [true] $($code)*);
    };
    ($data:tt current {$shdr:ident $src:tt} @is_vertex $($code:tt)*) => {
        glsl!($data current {$shdr $src} @push_bucket [false] $($code)*);
    };

    //add a string literal to the string for the current shader's source code
    ($data:tt {$str:expr} @src_str $($code:tt)*) => { glsl!($data current @bucket {$str} @src_str $($code)*); };
    ($data:tt current {$shdr:ident {$first:expr; $last:expr}} {$str:expr} @src_str $($code:tt)*) => {
        glsl!($data current {$shdr {$first; concat!($last, $str)}} @push_bucket $($code)*);
    };

    //add a runtime expression to the string for the current shader's source code
    ($data:tt {$str:expr} @src_expr $($code:tt)*) => { glsl!($data current @bucket {$str} @src_expr $($code)*); };
    ($data:tt current {$shdr:ident {$first:expr; $last:expr}} {$str:expr} @src_expr $($code:tt)*) => {
        glsl!($data current {$shdr {$first+$last+$str; ""}} @push_bucket $($code)*);
    };


    //
    //--------------------------------------------------------------------------------------------
    //The parsing of @ segments
    //--------------------------------------------------------------------------------------------
    //

    //
    //If we come accross a @Rust segment, then we should interpret it as actual rust code to
    //put into the current module
    //

    //we found another directive, so we should switch to that
    ($data:tt @shader @Rust @$directive:ident $($code:tt)*) => {
        glsl!($data @shader @$directive $($code)*);
    };

    //we found an item to add
    ($data:tt @shader @Rust $code:item $($rest:tt)*) => {
        $code
        glsl!($data @shader @Rust $($rest)*);
    };

    //we've gone through all of the code
    ($data:tt @shader @Rust) => {glsl!($data uni @bucket @push_bucket @create); };



    //
    //parse the next shader's type
    //

    ($data:tt @shader @Vertex $($code:tt)* ) => { glsl!($data [true] [] @set_flag Vertex @new_shdr @parse $($code)*); };
    ($data:tt @shader @TessControl $($code:tt)* ) => { glsl!($data TessControl @new_shdr @parse $($code)*); };
    ($data:tt @shader @TessEval $($code:tt)* ) => { glsl!($data TessEval @new_shdr @parse $($code)*); };
    ($data:tt @shader @Geometry $($code:tt)* ) => { glsl!($data Geometry @new_shdr @parse $($code)*); };
    ($data:tt @shader @Fragment $($code:tt)* ) => { glsl!($data [true] [,] @set_flag Fragment @new_shdr @parse $($code)*); };
    ($data:tt @shader @Compute $($code:tt)* ) => { glsl!($data [true] [,,] @set_flag Compute @new_shdr @parse $($code)*); };
    ($data:tt @shader @Lib $($code:tt)* ) => { glsl!($data Lib @new_shdr @parse $($code)*); };
    ($data:tt @shader @$err:ident $($code:tt)* ) => { compile_error!(concat!("Unrecognized GLSL shader type or macro directive: ", stringify!($err))); };
    ($data:tt @shader $err:tt $($code:tt)* ) => { compile_error!(concat!("Expected @[ShaderType], @Rust, or @Lib, found: ", stringify!($err))); };

    //
    //--------------------------------------------------------------------------------------------
    //Parse the current shader and it's lines of code
    //--------------------------------------------------------------------------------------------
    //

    //
    //we found a directive!
    //

    //glsl version
    ($data:tt @parse #version $val:tt $($code:tt)*) => {
        glsl!($data {concat!("#version ", stringify!($val), "\n")} @src_str @parse $($code)*);
    };

    //extension management
    ($data:tt @parse #extension $name:ident : $behavior:ident $($code:tt)*) => {
        glsl!($data {concat!("#extension ", stringify!($name:$behavior), "\n")} @src_str @parse $($code)*);
    };

    //C macros
    // ($data:tt @parse #define $name:ident($($args:tt)*) $expr:tt $($code:tt)* ) => {
    //     glsl!($data {concat!("#define ", glsl!(@stringify $name($($args)*) $expr), "\n")} @src_str @parse $($code)*);
    // };
    ($data:tt @parse #define $name:ident $expr:tt $($code:tt)* ) => {
        glsl!($data {concat!("#define ", glsl!(@stringify $name $expr), "\n")} @src_str @parse $($code)*);
    };


    //all other directives are invalid (yes, this does mean ALL of the C ones. You should be using rust macros instead :/ )
    ($data:tt @parse #undef $($code:tt)* ) => { compile_error!("#undef not supported (use Rust macros instead)");};
    ($data:tt @parse #ifndef $($code:tt)* ) => { compile_error!("#ifndef not supported (use Rust macros instead)");};
    ($data:tt @parse #if $($code:tt)* ) => { compile_error!("#if not supported (use Rust macros instead)");};
    ($data:tt @parse #elseif $($code:tt)* ) => { compile_error!("#elseif not supported (use Rust macros instead)");};
    ($data:tt @parse #endif $($code:tt)* ) => { compile_error!("#endif not supported (use Rust macros instead)");};
    ($data:tt @parse #include $($code:tt)* ) => { compile_error!("#include not supported (use Rust macros instead)");};
    ($data:tt @parse #pragma $($code:tt)* ) => { compile_error!("#pragma not supported (use Rust macros instead)");};
    ($data:tt @parse #line $($code:tt)* ) => { compile_error!("#line not supported (use Rust macros instead)");};
    ($data:tt @parse #$dir:ident $($code:tt)* ) => {
        compile_error!(concat!("Invalid glsl preprocessor macro: ", stringify!(#$dir)));
    };

    //
    //Importing code from other shaders
    //

    //if public, we want to pub use the struct or function
    ($data:tt @parse extern $($code:tt)*) => { glsl!($data {} @extern $($code)*); };
    ($data:tt @parse public extern $($code:tt)*) => { glsl!($data {pub} @extern $($code)*); };

    //for importing structs
    ($data:tt {$($kw:tt)*} @extern struct $name:path; $($code:tt)*) => {
        //$($kw)* use $name;
        glsl!($data {<$name as GLSLStruct>::SRC} @src_expr @parse $($code)*);
    };

    //for importing function
    ($data:tt {$($kw:tt)*} @extern $ret_type:ident $([$($s:tt)*])* $(::$name:ident)*($($params:tt)*); $($code:tt)*) => {
        glsl!([$ret_type $([$($s)*])*] @ty {[($($params)*)] @ty} @eval $data {$($kw)*} {$(::$name)*} @extern $($code)*);
    };
    ($data:tt {$($kw:tt)*} @extern $ret_type:ident $([$($s:tt)*])* $root:ident $(::$name:ident)*($($params:tt)*); $($code:tt)*) => {
        glsl!([$ret_type $([$($s)*])*] @ty {[($($params)*)] @ty} @eval $data {$($kw)*} {$root $(::$name)*} @extern $($code)*);
    };

    ([$ret_type:ty] [$params:ty] $data:tt {$($kw:tt)*} {$name:path} @extern $($code:tt)*) => {
        //$($kw)* use $name;
        glsl!($data {<$name as GLSLFunction<$ret_type, $params>>::SRC} @src_expr @parse $($code)*);
    };

    //
    //Structs
    //

    ($data:tt @parse $(#[$attr:meta])* public struct $name:ident {$($fields:tt)*} $($code:tt)*) => {
        glsl!(
            {$($fields)*} @decl_list $name {$($fields)*} {$(#[$attr])* pub} @struct
            $data
            {concat!(" ", glsl!(@stringify struct $name {$($fields)*}))} @src_str
            @parse $($code)*
        );
    };

    ($data:tt @parse $(#[$attr:meta])* uniform struct $name:ident {$($fields:tt)*} $($code:tt)*) => {
        glsl!(
            {$($fields)*} @decl_list $name {$($fields)*} {$(#[$attr])* pub} @struct
            $data
            {concat!(" ", glsl!(@stringify uniform struct $name {$($fields)*}))} @src_str
            $qualifier $name @var
            $($code)*
        );
    };

    ($data:tt @parse $(#[$attr:meta])* struct $name:ident {$($fields:tt)*} $($code:tt)*) => {
        glsl!(
            {$($fields)*} @decl_list $name {$($fields)*} {$(#[$attr])*} @struct
            $data
            {concat!(" ", glsl!(@stringify struct $name {$($fields)*}))} @src_str
            @parse $($code)*
        );
    };

    //
    //Functions (and exporting them to be used in other shaders)
    //

    ($data:tt @parse public void main() $($code:tt)*) => {compile_error!("Modifier \"public\" is not allowed on main()")};
    ($data:tt @parse public $ty:ident $name:ident($($params:tt)*) {$($content:tt)*} $($code:tt)*) => {

        #[allow(non_camel_case_types)]
        pub struct $name;

        glsl!(
            [($($params)*)] @ty {[$ty] @ty} @eval {glsl!(@stringify $ty $name($($params)*) {$($content)*})} $name @fun
            $data
            //[$name] fun @add_to @ignore
            {concat!(" ", glsl!(@stringify $ty $name($($params)*) {$($content)*}))} @src_str
            @parse $($code)*);
    };


    //
    //For parsing variables
    //

    //parse the next name (with possible array bounds) in the list and add it to the appropriate bucket

    //with a comma after
    ($data:tt $kind:ident $ty:tt @var $name:ident $([$($size:tt)*])*, $($code:tt)* ) => {
        glsl!($data
            {concat!(" ", stringify!($name $([$($size)*])*,))} @src_str
            [$ty $([$($size)*])*] $name @$kind $kind $ty @var
            $($code)*
        );
    };

    //with a semicolon after
    ($data:tt $kind:ident $ty:tt @var $name:ident $([$($size:tt)*])*; $($code:tt)* ) => {
        glsl!($data
            {concat!(" ", stringify!($name $([$($size)*])*;), "\n")} @src_str
            [$ty $([$($size)*])*] $name @$kind
            @parse $($code)*
        );
    };

    //with an expression after
    ($data:tt $kind:ident $ty:tt @var $name:ident $([$($size:tt)*])* = $init:expr, $($code:tt)* ) => {
        glsl!($data
            {concat!(" ", stringify!($name $([$($size)*])* = $init,))} @src_str
            [$ty $([$($size)*])*] $name @$kind $kind $ty @var
            $($code)*
        );
    };
    ($data:tt $kind:ident $ty:tt @var $name:ident $([$($size:tt)*])* = $init:expr; $($code:tt)* ) => {
        glsl!($data
            {concat!(" ", stringify!($name $([$($size)*])* = $init;), "\n")} @src_str
            [$ty $([$($size)*])*] $name @$kind
            @parse $($code)*
        );
    };

    //
    //Finding bind points, uniforms, and attributes
    //

    //we found a uniform with a struct type
    ($data:tt @parse uniform struct {$($fields:tt)*} $name:ident $($code:tt)*) => {
        glsl!($data {concat!(" ", glsl!(@stringify uniform struct {$($fields)*}))} @src_str uniform {$($fields)*} @var $name $($code)*);
    };

    //we found a uniform or shader storage block!
    ($data:tt @parse uniform $Name:ident {$($fields:tt)*} $($code:tt)*) => {
        glsl!($data
            {concat!(glsl!(@stringify uniform $Name {$($fields)*}))} @src_str
            layout() [{$($fields)*}] $Name uniform @block
            @parse $($code)*
        );
    };
    ($data:tt @parse layout($($params:tt)*) uniform $Name:ident {$($fields:tt)*} $($code:tt)*) => {
        glsl!($data
            {concat!(glsl!(@stringify layout($($params)*) uniform $Name {$($fields)*}))} @src_str
            layout($($params)*) @block_layout [{$($fields)*}] $Name uniform @block
            @parse $($code)*
        );
    };
    ($data:tt @parse buffer $Name:ident {$($fields:tt)*} $($code:tt)*) => {
        glsl!($data
            {concat!(glsl!(@stringify buffer $Name {$($fields)*}))} @src_str
            layout() [{$($fields)*}] $Name buffer @block
            @parse $($code)*
        );
    };
    ($data:tt @parse layout($($params:tt)*) buffer $Name:ident {$($fields:tt)*} $($code:tt)*) => {
        glsl!($data
            {concat!(glsl!(@stringify layout($($params)*) buffer $Name {$($fields)*}))} @src_str
            layout($($params)*) @block_layout [{$($fields)*}] $Name buffer @block
            @parse $($code)*
        );
    };
    ($data:tt @parse layout($($params:tt)*) $mod1:ident buffer $Name:ident {$($fields:tt)*} $($code:tt)*) => {
        glsl!($data
            {concat!(glsl!(@stringify layout($($params)*) $mod1 buffer $Name {$($fields)*}))} @src_str
            layout($($params)*) @block_layout [{$($fields)*}] $Name buffer @block
            @parse $($code)*
        );
    };
    ($data:tt @parse layout($($params:tt)*) $mod1:ident $mod2:ident buffer $Name:ident {$($fields:tt)*} $($code:tt)*) => {
        glsl!($data
            {concat!(glsl!(@stringify layout($($params)*) $mod1 $mod2 buffer $Name {$($fields)*}))} @src_str
            layout($($params)*) @block_layout [{$($fields)*}] $Name buffer @block
            @parse $($code)*
        );
    };

    //we found a uniform!
    ($data:tt @parse uniform $ty:ident $name:ident $($code:tt)*) => {
        glsl!($data {concat!(" ", stringify!(uniform $ty))} @src_str uniform $ty @var $name $($code)*);
    };

    //we found an attribute!
    ($data:tt @parse attribute $ty:ident $name:ident $($code:tt)*) => {
        glsl!($data {concat!(" ", stringify!(attribute $ty))} @src_str attribute $ty @var $name $($code)*);
    };

    //we found a possible attribute! (we need to make sure we're currently processing the vertex shader tho)
    ($data:tt @parse in $ty:ident $name:ident $($code:tt)*) => {
        glsl!(
            $data @is_vertex @if {
                {concat!(" ", stringify!(in $ty))} @src_str attribute $ty @var $name
            } @then {
                {concat!(" ", stringify!(in $ty $name))} @src_str @parse
            } @else $($code)*
        );
    };

    //we found the next code segment!
    ($data:tt @parse @$next:ident $($code:tt)*) => {
        glsl!($data @src @shader @$next $($code)*);
    };

    //add a newline for every semicolon so we get proper line numbers
    ($data:tt @parse ; $($code:tt)*) => {
        glsl!($data {";\n"} @src_str @parse $($code)*);
    };

    //just move the next token into the source bucket
    ($data:tt @parse $t:tt $($code:tt)*) => {
        glsl!($data {glsl!(@stringify $t)} @src_str @parse $($code)*);
    };

    //if there's no more code, add the src for this shader, reorder the buckets, and create the Program
    ($data:tt @parse ) => { glsl!($data @src uni @bucket @push_bucket @create); };


    //
    //The main loop, module creator, and system initialization
    //

    //for when there are no more programs to make
    ($d:tt) => {};



    //start the parsing of a shader and put it in a module
    //$d is a '$' token that MUST be passed from top level so that we can create database macros in order to track duplicate uniforms
    ($d:tt pub mod $name:ident {$($code:tt)*} $($rest:tt)*) => {glsl!($d @create_mod {pub} mod $name {$($code)*} $($rest)*);};
    ($d:tt mod $name:ident {$($code:tt)*} $($rest:tt)*) => {glsl!($d @create_mod {} mod $name {$($code)*} $($rest)*);};
    ($d:tt @create_mod {$($kw:tt)*} mod $name:ident {$($code:tt)*} $($rest:tt)*) => {

        #[allow(non_snake_case)]
        $($kw)* mod $name {
            #[allow(unused_imports)] use std::result::Result;
            #[allow(unused_imports)] use $crate::glsl_type::*;
            #[allow(unused_imports)] use $crate::*;

            //NOTE: yes, I am aware I am making a macro that makes macros inside of a macro
            //The reason for defining them here is so that they don't clutter the namespace and are confined
            //to this special purpose module. But yes, this is kind of awful... :( There is a special hell for
            //people like me...

            //a macro for making macros to test if two uniforms are named the same thing
            //this is incredibly important since different shader stages might need to use the same
            //uniform and will thus declare it more than once
            #[doc(hidden)]
            #[allow(unused_macros)]
            macro_rules! _register_name {
                //this pattern declares $D and $k where $D is the dollar sign tt for this macro-maker
                //and $k is the uniform name we want to register.
                //Writing directly, this would look like:
                //($D:tt $k:tt) => {..}
                ($d D:tt  $d k:tt) => {

                    //here, we create a macro with the name of the uniform that will take in
                    //an ident, a callback, and a code stack and pass "true" or "false" to callback
                    //depending on if the ident matches the uniform name
                    #[doc(hidden)]
                    #[allow(unused_macros)]
                    macro_rules! $k {

                        //yup... a dollar sign is now "$d D" :/

                        //this matches the uniform name
                        //Writing directly, this would look like:
                        //([uniform_name] $callback:ident $($code:tt)*) => {..}
                        ($d k  $d D callback:ident  $d D( $d D code:tt )*) => {
                            $d D callback!([true] $d D( $d D code )*);
                        };

                        //This matches names that DONT match the uniform name
                        //Writing directly, this would look like:
                        //($t:tt $callback:ident $($code:tt)*) => {..}
                        ($d D t:tt  $d D callback:ident  $d D( $d D code:tt )*) => {
                            $d D callback!([false] $d D( $d D code )*);
                        };
                    }
                }
            }

            //start the processing with 3 data buckets (uniforms, attributes, and source code)
            glsl!(($d @uni {} @attr {} @block {} @src {} @flags {[false] [false] [false]} @fun {} @current {} ) @shader $($code)*);
        }

        //get the other shaders (if any)
        glsl!($d $($rest)*);
    };

    //
    //Create the rust-side version of a glsl function
    //

    ([$param:ty] [$ret:ty] {$src:expr} $name:ident @fun $($code:tt)*) => {
        unsafe impl GLSLFunction<$ret, $param> for $name {
            const SRC: &'static str = $src;
        }
        glsl!($($code)*);
    };

    //
    //Create the rust-side version of a glsl struct
    //

    (@struct_first_name $var:ident {[$name:ident: $ty:ty] $($rest:tt)*}) => {
        <$ty as GLSLType>::first_element_name($var + "." + stringify!($name))
    };

    ({$([$name:ident: $ty:ty])*} $struct_name:ident {$($src:tt)*} {$($mod:tt)*} @struct $($code:tt)*) => {
        #[repr(C)]
        #[derive(Clone, Copy, PartialEq, Debug, Default)]
        $($mod)* struct $struct_name {
            $(pub $name: $ty),*
        }

        unsafe impl AlignedVec4 for $struct_name where ($($ty),*): AlignedVec4 {}
        unsafe impl Layout<std140> for $struct_name where Self:AlignedVec4, $($ty: Layout<std140>),* {}
        unsafe impl Layout<std430> for $struct_name where $($ty: Layout<std430>),* {}

        //methods for getting attribute arrays from buffers
        impl $struct_name where $($ty: GLSLType + GLSLData<$ty>),* {

            #[inline]
            pub fn get_attrib_arrays<'a, A:BufferAccess>(buf: &'a Buffer<[Self], A>) -> ($(AttribArray<'a, $ty>),*) {
                unsafe {
                    use std::mem::*;
                    let uninit: Self = uninitialized();
                    let start: *const u8 = transmute(&uninit);
                    let arrays = (
                        $(buf.get_attrib_array::<$ty, $ty>(i8::wrapping_sub(transmute::<&$ty,*const u8>(&uninit.$name) as i8, start as i8) as usize)),*
                    );
                    forget(uninit);

                    arrays
                }
            }

            #[inline]
            pub fn get_attributes<'a, A:BufferAccess>(buf: &'a Buffer<[Self], A>) -> ($(Attribute<'a, $ty>),*) {
                let ($($name),*) = Self::get_attrib_arrays(buf);
                ($(Attribute::Array($name)),*)
            }

        }


        unsafe impl GLSLStruct for $struct_name {
            const SRC: &'static str = stringify!(struct $struct_name {$($src)*};);
        }

        unsafe impl GLSLType for $struct_name where $($ty: GLSLType),* {
            type AttributeFormat = UnsupportedFormat;

            unsafe fn load_uniforms(id: GLint, data: &[Self]) {

                for x in data {
                    #[allow(unused_variables)]
                    #[allow(unused_mut)]
                    let mut i = id;

                    $(
                        <$ty as GLSLType>::load_uniform(i, &x.$name);
                        *(&mut i) = {i + <$ty as GLSLType>::uniform_locations() as GLint};
                    )*
                }


            }

            unsafe fn get_uniform(p: GLuint, id:GLint) -> Self {
                let mut value = ::std::mem::uninitialized::<Self>();

                #[allow(unused_variables)]
                #[allow(unused_mut)]
                let mut i = id;

                $(
                    value.$name = <$ty as GLSLType>::get_uniform(p, i);
                    *(&mut i) = {i + <$ty as GLSLType>::uniform_locations() as GLint};
                )*

                value
            }

            #[inline]
            fn uniform_locations() -> GLuint {
                0 $( + <$ty as GLSLType>::uniform_locations())*
            }

            #[inline]
            fn first_element_name(var: String) -> String {
                glsl!(@struct_first_name var {$([$name: $ty])*})
            }
        }

        glsl!($($code)*);

    };


    //
    //create the module and struct and implement the Program trait
    //

    (($d:tt
        @uni {$([$uname:ident: $u_ty:ty])*}
        @attr {$([$aname:ident: $a_ty:ty])*}
        @block {$([$block:ident<$I:ident, $L:ident, $A:ident>: $b_ty:ty])*}
        @src {$([$shdr:ident=$src:expr])*}
        @flags {[$vert:tt] [$frag:tt] [$compute:tt]}
        $($ignore:tt)*
    ) @create) =>
    {

        glsl!{[$vert] [$frag] @and [$compute] @or @if {[

            //note: Program::resource, Program::uniform, and Program::subroutine are named as such
            //in part because those identifiers are reserved keywords in GLSL, and so, we don't
            //need to worry about a name collision with any of the other fields


            pub struct Program {
                resource: ProgramID,
                $($aname: AttributeLocation,)*
                $($block: $I<$L, $b_ty>,)*

                #[allow(dead_code)]
                uniform: [UniformLocation; glsl!([$($uname)*] @count @quote)],
                $(pub $uname: Uniform<$u_ty>,)*
            }

            #[inline]
            pub fn init(context: &GLProvider) -> Result<self::Program, GLError> {
                <self::Program as $crate::program::Program>::init(context)
            }

            unsafe impl $crate::program::Program for self::Program {

                fn init(context: &GLProvider) -> Result<Self, GLError> {
                    #[allow(unused_unsafe)]
                    unsafe {

                        #[allow(unused_variables)]
                        #[allow(unused_mut)]
                        let mut i = 0;


                        let p = ProgramID::from_source(context, vec![$(($src, ShaderType::$shdr)),*])?;
                        let uniforms =
                        [$(
                            match UniformLocation::get(
                                &p, <$u_ty as GLSLType>::first_element_name(stringify!($uname).to_owned()).as_str()
                            ) {
                                Ok(loc) => loc,
                                Err(loc) => loc
                            }
                        ),*];

                        #[allow(unused_mut)]
                        let mut program = self::Program {

                            $(
                                $aname: match AttributeLocation::get(&p, stringify!($aname)) {
                                    Ok(loc) => loc,
                                    Err(loc) => loc
                                },
                            )*

                            $($block: $I::get(&p, stringify!($block)),)*

                            $(
                                $uname: {
                                    let j = i;
                                    *&mut i = i+1;
                                    uniforms[j].get_uniform()
                                },
                            )*

                            uniform: uniforms,
                            resource: p
                        };

                        #[allow(unused_variables)]
                        #[allow(unused_mut)]
                        let mut j = 0;
                        $(
                            program.$block.set_binding(j);
                            *&mut j = j+1;
                        )*

                        Ok(program)
                    }
                }

            }

            impl self::Program {

                #[inline]
                unsafe fn load_uniforms(&self) {
                    #[allow(unused_variables)]
                    #[allow(unused_mut)]
                    let mut i = 0;
                    $(
                        self.uniform[i].load(&self.$uname);
                        *&mut i = i+1;
                    )*
                }

                glsl! {
                    [$vert] [$frag] @and @if {[
                        pub fn draw<'b $(,$A: BufferAccess)*>(
                            &self,
                            _context: &mut Context,
                            mode: DrawMode,
                            count: usize,
                            $($block: &Buffer<$b_ty, $A>,)*
                            $($aname: Attribute<'b, $a_ty>),*
                        )
                        {
                            unsafe {
                                //make sure the uniforms are loaded onto the gpu
                                self.resource.use_program();
                                self.load_uniforms();
                                $(self.$aname.load(&$aname);)*
                                $(self.$block.bind_buffer_range($block);)*

                                $crate::gl::Flush();
                                $crate::gl::Finish();

                                // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                                // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                                $crate::gl::DrawArrays(mode as GLenum, 0, count as GLsizei);
                                // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                                // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

                                $(self.$block.unbind();)*
                                ProgramID::unbind_program();
                            }
                        }] @quote
                    } @then {@ret} @else
                }

                glsl! {
                    [$compute] @if {[
                        pub fn compute<$($A: BufferAccess,)*>(
                            &self,
                            count_x: GLuint, count_y: GLuint, count_z: GLuint,
                            $($block: &mut Buffer<$b_ty, $A>),*
                        )
                        {
                            unsafe {
                                //make sure the uniforms are loaded onto the gpu
                                self.resource.use_program();
                                self.load_uniforms();
                                $(self.$block.bind_buffer_range($block);)*

                                // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                                // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                                $crate::gl::DispatchCompute(count_x, count_y, count_z);
                                // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
                                // !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!


                                $crate::gl::MemoryBarrier($crate::gl::ALL_BARRIER_BITS);

                                $(self.$block.unbind();)*
                                ProgramID::unbind_program();
                            }
                        }] @quote
                    } @then {@ret} @else
                }
            }] @quote
        } @then {@ret} @else }

    };



    //catch an error
    (@ $kw:ident $($code:tt)*) => { compile_error!(concat!(stringify!($kw)," is an invalid directive for the glsl! macro.")) };

    //so that we can write straight rust code
    ($d:tt $rust_code:item $($rest:tt)*) => {
        $rust_code
        glsl!($d $($rest)*);
    };

}
