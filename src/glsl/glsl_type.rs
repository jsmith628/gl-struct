
use std::slice::*;

///
///A macro constucting the gl functions for managing uniforms using the
///concat_idents! macro. Note that this can only be used in expressions,
///and as such, you must both import the function name into the current module
///AND borrow the function as a pointer in order to use.
///
///Also, while this only does glUniform* stuff, in the future I may expand it if
///it would be of use.
///
macro_rules! gl_builder {

    ({$($gl:ident)*} c_bool @ty_suffix $($tail:tt)*) => { gl_builder!({$($gl)* ui} $($tail)*) };
    ({$($gl:ident)*} GLuint @ty_suffix $($tail:tt)*) => { gl_builder!({$($gl)* ui} $($tail)*) };
    ({$($gl:ident)*} GLint @ty_suffix $($tail:tt)*) => { gl_builder!({$($gl)* i} $($tail)*) };
    ({$($gl:ident)*} GLfloat @ty_suffix $($tail:tt)*) => { gl_builder!({$($gl)* f} $($tail)*) };
    ({$($gl:ident)*} GLdouble @ty_suffix $($tail:tt)*) => { gl_builder!({$($gl)* d} $($tail)*) };

    ({$($gl:ident)*} 1 @uni_vec $($tail:tt)*) => { gl_builder!({$($gl)* Uniform1} $($tail)*) };
    ({$($gl:ident)*} 2 @uni_vec $($tail:tt)*) => { gl_builder!({$($gl)* Uniform2} $($tail)*) };
    ({$($gl:ident)*} 3 @uni_vec $($tail:tt)*) => { gl_builder!({$($gl)* Uniform3} $($tail)*) };
    ({$($gl:ident)*} 4 @uni_vec $($tail:tt)*) => { gl_builder!({$($gl)* Uniform4} $($tail)*) };

    ({$($gl:ident)*} 2 @uni_mat $($tail:tt)*) => { gl_builder!({$($gl)* UniformMatrix2} $($tail)*) };
    ({$($gl:ident)*} 3 @uni_mat $($tail:tt)*) => { gl_builder!({$($gl)* UniformMatrix3} $($tail)*) };
    ({$($gl:ident)*} 4 @uni_mat $($tail:tt)*) => { gl_builder!({$($gl)* UniformMatrix4} $($tail)*) };

    ({$($gl:ident)*} 2 @mat_xN $($tail:tt)*) => { gl_builder!({$($gl)* x2} $($tail)*) };
    ({$($gl:ident)*} 3 @mat_xN $($tail:tt)*) => { gl_builder!({$($gl)* x3} $($tail)*) };
    ({$($gl:ident)*} 4 @mat_xN $($tail:tt)*) => { gl_builder!({$($gl)* x4} $($tail)*) };

    ({$($gl:ident)*} @v $($tail:tt)*) => { gl_builder!({$($gl)* v} $($tail)*) };

    ({$($gl:ident)*} @concat) => { concat_idents!($($gl),*) };

    (@get $prim:ident) => { gl_builder!({GetUniform} $prim @ty_suffix @v @concat) };
    (@get [$prim:ident; $c:tt]) => { gl_builder!(@get $prim) };
    (@get [[$prim:ident; $c1:tt]; $c2:tt]) => { gl_builder!(@get $prim) };

    (@set $prim:ident) => { gl_builder!(@set [$prim; 1]) };
    (@set [$prim:ident; $c:tt]) => { gl_builder!({} $c @uni_vec $prim @ty_suffix @v @concat) };
    (@set [[$prim:ident; $c1:tt]; $c2:tt]) => {
        macro_program! (
            [$c1] @hex {[$c2] @hex} @eval @eq
            { gl_builder @pass_to {} $c2 @uni_mat }
            { gl_builder @pass_to {} $c2 @uni_mat $c1 @mat_xN }
            @if @eval
            $prim @ty_suffix @v @concat
        )
    };



}

macro_rules! glsl_type {

    //
    //In order to make all the primitives and the like fit the std430 layout
    //(and allow type checking for std140), we need to set the alignment of each type accordingly
    //

    //all scalars besides double (which has 8) require an alignment of 4bytes, fitstd140,
    //but they cannot satisfy sdt140 if they are in an array
    (GLdouble @align $($tail:tt)*) => { glsl_type!({8} false true true $($tail)*); };
    ($prim:ident @align $($tail:tt)*) => { glsl_type!({4} false true true $($tail)*); };

    //for vectors, the alignment is 2N for vec2 and 4N for vec3 and vec4, and hence,
    //all BUT vec2/ivec2/uvec2/bvec2 can be put into std140 arrays
    ({4} false true true 2 @align_vec $($tail:tt)*) => { glsl_type!({8} false $($tail)*); };
    ({8} false true true 2 @align_vec $($tail:tt)*) => { glsl_type!({16} true $($tail)*); }; //dvec2's have an alignent equal to that of vec4
    ({4} false true true 3 @align_vec $($tail:tt)*) => { glsl_type!({16} true $($tail)*); };
    ({8} false true true 3 @align_vec $($tail:tt)*) => { glsl_type!({32} true $($tail)*); };
    ({4} false true true 4 @align_vec $($tail:tt)*) => { glsl_type!({16} true $($tail)*); };
    ({8} false true true 4 @align_vec $($tail:tt)*) => { glsl_type!({32} true $($tail)*); };

    //all vecs are std140 complient
    ([$prim:ident; $c:tt] @align $($tail:tt)*) => { glsl_type!($prim @align $c @align_vec true true $($tail)*); };

    //matrices are considered as arrays over their columns, so all matNx2 and matNx4 are std430 and all matNx4 are 140,
    //However, since we're storing matNx3's as [[T; 3]; N], the columns are not vec4 aligned, so they aren't std430 OR std140
    ([[GLfloat; 2]; $c2:tt] @align $($tail:tt)*) => { glsl_type!({8} false false true $($tail)*); };
    ([[$prim:ident; 3]; $c2:tt] @align $($tail:tt)*) => { glsl_type!({8} false false false $($tail)*); };
    ([[$prim:ident; $c1:tt]; $c2:tt] @align $($tail:tt)*) => { glsl_type!($prim @align $c1 @align_vec true true $($tail)*); };

    //determine if the type is a scalar or matrix or neither
    ({$a:expr} $b1:tt $b2:tt $b3:tt $prim:ident @type $($tail:tt)*) => { glsl_type!({$a} $b1 $b2 $b3 true false $($tail)*); };
    ({$a:expr} $b1:tt $b2:tt $b3:tt [$prim:ident; $c:tt] @type $($tail:tt)*) => { glsl_type!({$a} $b1 $b2 $b3 false false $($tail)*); };
    ({$a:expr} $b1:tt $b2:tt $b3:tt [[$prim:ident; $c1:tt]; $c2:tt] @type $($tail:tt)*) => { glsl_type!({$a} $b1 $b2 $b3 false true $($tail)*); };

    //the initial macro call
    ({$fmt:ty} $name:ident = $($ty:tt)*) => {
        glsl_type!($($ty)* @align $($ty)* @type {$fmt} {$($ty)*} {gl_builder!(@set $($ty)*)} {gl_builder!(@get $($ty)*)} $name);
        glsl_type!(@index $name = $($ty)*);
    };

    (@index $name:ident = $prim:ident) => {};
    (@index $name:ident = [$T:ty; $c:tt]) => {
        impl Index<usize> for $name {
            type Output = $T;
            #[inline] fn index(&self, i: usize) -> &$T { &self.value[i] }
        }

        impl IndexMut<usize> for $name {
            #[inline] fn index_mut(&mut self, i: usize) -> &mut $T { &mut self.value[i] }
        }
    };

    ({$a:expr} $align_vec4:tt $std140:tt $std430:tt $scalar:tt $mat:tt {$fmt:ty} {$prim:ty} {$set:expr} {$get:expr} $name:ident) => {

        macro_program! {
            [$scalar] @not [
                #[repr(C)]
                #[repr(align($a))]
                #[derive(Clone, Copy, PartialEq, Debug, Default)]
                #[allow(non_camel_case_types)]
                pub struct $name {
                    pub value: $prim
                }

                impl From<$prim> for $name { #[inline] fn from(v: $prim) -> Self { $name{value: v} } }
                impl From<$name> for $prim { #[inline] fn from(v: $name) -> Self { v.value } }

                impl<G:GLSLType> AttributeData<G> for $name where $prim: AttributeData<G> {
                    #[inline] fn format() -> G::AttributeFormat { <$prim as AttributeData<G>>::format() }
                }
            ]
            [ #[allow(non_camel_case_types)] pub type $name = $prim; ]
            @if @quote
        }

        unsafe impl GLSLType for $name {
            type AttributeFormat = $fmt;

            unsafe fn load_uniforms(id: GLint, data: &[Self]){
                let f = &$set;
                macro_program!{
                    [$mat]
                    [f(id, data.len() as GLint, false as GLboolean, transmute(&data[0][0][0]));]
                    [f(id, data.len() as GLint, transmute(&data[0]));]
                    @if @quote
                }
            }

            unsafe fn get_uniform(p: GLuint, id:GLint) -> Self {
                let mut data = ::std::mem::uninitialized::<Self>();
                let f = &$get;
                f(p, id, transmute(&mut data));
                data
            }
        }

        macro_program!{
            [$std430] [unsafe impl Layout<std430> for $name {}] [] @if @quote
        }

        macro_program!{
            [$std140] [unsafe impl Layout<std140> for $name {}] [] @if @quote
        }

        macro_program!{
            [$align_vec4] [unsafe impl AlignedVec4 for $name {}] [] @if @quote
        }


    };

}

//we want to throw everything in a module since these are kindof really common data type names
//and we're kinda not 100% following rust naming convention, so we don't want literally everything
//to get pulled into the crate module by default

pub use gl::types::*;
use gl::*;
use ::*;

use std::mem::transmute;
use std::ops::*;

#[repr(align(4))]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Default)]
pub struct c_bool(GLuint);

macro_rules! impl_c_bool {
    ($($Trait:ident.$fun:ident $op:tt),*) => {$(
        impl $Trait<Self> for c_bool { type Output = Self; #[inline] fn $fun(self, r:Self) -> Self {c_bool(self.0 $op r.0)} }
        impl $Trait<bool> for c_bool { type Output = Self; #[inline] fn $fun(self, r:bool) -> Self {c_bool(self.0 $op r as u32)} }
        impl $Trait<c_bool> for bool { type Output = c_bool; #[inline] fn $fun(self, r:c_bool) -> c_bool {c_bool(self as u32 $op r.0)} }
    )*}
}

macro_rules! impl_c_bool_assign {
    ($($Trait:ident.$fun:ident $op:tt),*) => {$(
        impl $Trait<Self> for c_bool { #[inline] fn $fun(&mut self, r:Self) {self.0 $op r.0;} }
        impl $Trait<bool> for c_bool { #[inline] fn $fun(&mut self, r:bool) {self.0 $op r as u32;} }
        impl $Trait<c_bool> for bool { #[inline] fn $fun(&mut self, r:c_bool) {*self $op r.0>0;} }
    )*}
}

impl_c_bool!(BitAnd.bitand &, BitOr.bitor |, BitXor.bitxor &);
impl_c_bool_assign!(BitAndAssign.bitand_assign &=, BitOrAssign.bitor_assign |=, BitXorAssign.bitxor_assign &=);

impl From<bool> for c_bool { #[inline] fn from(b: bool) -> Self {c_bool(b as GLuint)} }
impl From<c_bool> for bool { #[inline] fn from(b: c_bool) -> Self {b.0>0} }
impl From<GLuint> for c_bool { #[inline] fn from(b: GLuint) -> Self {c_bool(b)} }
impl From<c_bool> for GLuint { #[inline] fn from(b: c_bool) -> Self {b.0} }
impl From<GLboolean> for c_bool { #[inline] fn from(b: GLboolean) -> Self {c_bool(b as GLuint)} }
impl From<c_bool> for GLboolean { #[inline] fn from(b: c_bool) -> Self {b.0 as GLboolean} }

#[allow(non_camel_case_types)]
pub type void = ();

//booleans
glsl_type!({IntFormat} gl_bool = c_bool);
glsl_type!({IVecFormat} bvec2 = [c_bool; 2]);
glsl_type!({IVecFormat} bvec3 = [c_bool; 3]);
glsl_type!({IVecFormat} bvec4 = [c_bool; 4]);

//integers
glsl_type!({IntFormat} int = GLint);
glsl_type!({IVecFormat} ivec2 = [GLint; 2]);
glsl_type!({IVecFormat} ivec3 = [GLint; 3]);
glsl_type!({IVecFormat} ivec4 = [GLint; 4]);

//unsigned integers
glsl_type!({IntFormat} uint = GLuint);
glsl_type!({IVecFormat} uvec2 = [GLuint; 2]);
glsl_type!({IVecFormat} uvec3 = [GLuint; 3]);
glsl_type!({IVecFormat} uvec4 = [GLuint; 4]);

//floats
glsl_type!({FloatFormat} float = GLfloat);
glsl_type!({VecFormat} vec2 = [GLfloat; 2]);
glsl_type!({VecFormat} vec3 = [GLfloat; 3]);
glsl_type!({VecFormat} vec4 = [GLfloat; 4]);
glsl_type!({[VecFormat; 2]} mat2   = [[GLfloat; 2]; 2]);
glsl_type!({[VecFormat; 2]} mat2x3 = [[GLfloat; 3]; 2]);
glsl_type!({[VecFormat; 2]} mat2x4 = [[GLfloat; 4]; 2]);
glsl_type!({[VecFormat; 3]} mat3x2 = [[GLfloat; 2]; 3]);
glsl_type!({[VecFormat; 3]} mat3   = [[GLfloat; 3]; 3]);
glsl_type!({[VecFormat; 3]} mat3x4 = [[GLfloat; 4]; 3]);
glsl_type!({[VecFormat; 4]} mat4x2 = [[GLfloat; 2]; 4]);
glsl_type!({[VecFormat; 4]} mat4x3 = [[GLfloat; 3]; 4]);
glsl_type!({[VecFormat; 4]} mat4   = [[GLfloat; 4]; 4]);

//doubles
glsl_type!({DoubleFormat} double = GLdouble);
glsl_type!({DVecFormat} dvec2 = [GLdouble; 2]);
glsl_type!({DVecFormat} dvec3 = [GLdouble; 3]);
glsl_type!({DVecFormat} dvec4 = [GLdouble; 4]);
glsl_type!({[DVecFormat; 2]} dmat2   = [[GLdouble; 2]; 2]);
glsl_type!({[DVecFormat; 2]} dmat2x3 = [[GLdouble; 3]; 2]);
glsl_type!({[DVecFormat; 2]} dmat2x4 = [[GLdouble; 4]; 2]);
glsl_type!({[DVecFormat; 3]} dmat3x2 = [[GLdouble; 2]; 3]);
glsl_type!({[DVecFormat; 3]} dmat3   = [[GLdouble; 3]; 3]);
glsl_type!({[DVecFormat; 3]} dmat3x4 = [[GLdouble; 4]; 3]);
glsl_type!({[DVecFormat; 4]} dmat4x2 = [[GLdouble; 2]; 4]);
glsl_type!({[DVecFormat; 4]} dmat4x3 = [[GLdouble; 3]; 4]);
glsl_type!({[DVecFormat; 4]} dmat4   = [[GLdouble; 4]; 4]);


macro_rules! impl_array_type {

    ($attrib_support:tt $($num:tt)*) => {
        $(
            unsafe impl<T:GLSLType> GLSLType for [T; $num] {
                macro_program!{
                    [$attrib_support]
                        [type AttributeFormat = [T::AttributeFormat; $num];]
                        [type AttributeFormat = UnsupportedFormat;]
                    @if @quote
                }

                unsafe fn load_uniforms(id: GLint, data: &[Self]){
                    let flattened = from_raw_parts(&data[0][0] as *const T, data.len() * $num);
                    T::load_uniforms(id, flattened);
                }

                unsafe fn get_uniform(p: GLuint, id:GLint) -> Self {
                    let mut data = ::std::mem::uninitialized::<Self>();
                    for i in 0..$num {
                        data[i] = T::get_uniform(p, id + i as GLint);
                    }
                    data
                }

                #[inline] fn uniform_locations() -> GLuint { T::uniform_locations() * $num }
                #[inline] fn first_element_name(var: String) -> String { T::first_element_name(var + "[0]") }

            }

            unsafe impl<T:AlignedVec4> AlignedVec4 for [T; $num] {}
            unsafe impl<T:AlignedVec4+Layout<std140>> Layout<std140> for [T; $num] {}
            unsafe impl<T:Layout<std430>> Layout<std430> for [T; $num] {}

        )*
    }


}

impl_array_type!{ true
    01 02 03 04 05 06 07 08 09 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
}

//As much as it looks like it, this *actually* isn't overkill.
//It is *very* possible that you may need mat4 uniform arrays for something
//such as storing bone transforms for skeletal animation for example.
//Of course, at some point, the user *could* (and should) just use
//arrays of arrays to get more indices (though that needs GL 4)
#[cfg(any(feature = "large_uniform_arrays", feature = "extra_large_uniform_arrays"))]
impl_array_type!{ false
    033 034 035 036 037 038 039 040 041 042 043 044 045 046 047 048 049 050 051 052 053 054 055 056 057 058 059 060 061 062 063 064
    065 066 067 068 069 070 071 072 073 074 075 076 077 078 079 080 081 082 083 084 085 086 087 088 089 090 091 092 093 094 095 096
    097 098 099 100 101 102 103 104 105 106 107 108 109 110 111 112 113 114 115 116 117 118 119 120 121 122 123 124 125 126 127 128
    129 130 131 132 133 134 135 136 137 138 139 140 141 142 143 144 145 146 147 148 149 150 151 152 153 154 155 156 157 158 159 160
    161 162 163 164 165 166 167 168 169 170 171 172 173 174 175 176 177 178 179 180 181 182 183 184 185 186 187 188 189 190 191 192
    193 194 195 196 197 198 199 200 201 202 203 204 205 206 207 208 209 210 211 212 213 214 215 216 217 218 219 220 221 222 223 224
    225 226 227 228 229 230 231 232 233 234 235 236 237 238 239 240 241 242 243 244 245 246 247 248 249 250 251 252 253 254 255 256
}

#[cfg(feature = "extra_large_uniform_arrays")]
impl_array_type! { false
    257 258 259 260 261 262 263 264 265 266 267 268 269 270 271 272 273 274 275 276 277 278 279 280 281 282 283 284 285 286 287 288
    289 290 291 292 293 294 295 296 297 298 299 300 301 302 303 304 305 306 307 308 309 310 311 312 313 314 315 316 317 318 319 320
    321 322 323 324 325 326 327 328 329 330 331 332 333 334 335 336 337 338 339 340 341 342 343 344 345 346 347 348 349 350 351 352
    353 354 355 356 357 358 359 360 361 362 363 364 365 366 367 368 369 370 371 372 373 374 375 376 377 378 379 380 381 382 383 384
    385 386 387 388 389 390 391 392 393 394 395 396 397 398 399 400 401 402 403 404 405 406 407 408 409 410 411 412 413 414 415 416
    417 418 419 420 421 422 423 424 425 426 427 428 429 430 431 432 433 434 435 436 437 438 439 440 441 442 443 444 445 446 447 448
    449 450 451 452 453 454 455 456 457 458 459 460 461 462 463 464 465 466 467 468 469 470 471 472 473 474 475 476 477 478 479 480
    481 482 483 484 485 486 487 488 489 490 491 492 493 494 495 496 497 498 499 500 501 502 503 504 505 506 507 508 509 510 511 512
    513 514 515 516 517 518 519 520 521 522 523 524 525 526 527 528 529 530 531 532 533 534 535 536 537 538 539 540 541 542 543 544
    545 546 547 548 549 550 551 552 553 554 555 556 557 558 559 560 561 562 563 564 565 566 567 568 569 570 571 572 573 574 575 576
    577 578 579 580 581 582 583 584 585 586 587 588 589 590 591 592 593 594 595 596 597 598 599 600 601 602 603 604 605 606 607 608
    609 610 611 612 613 614 615 616 617 618 619 620 621 622 623 624 625 626 627 628 629 630 631 632 633 634 635 636 637 638 639 640
    641 642 643 644 645 646 647 648 649 650 651 652 653 654 655 656 657 658 659 660 661 662 663 664 665 666 667 668 669 670 671 672
    673 674 675 676 677 678 679 680 681 682 683 684 685 686 687 688 689 690 691 692 693 694 695 696 697 698 699 700 701 702 703 704
    705 706 707 708 709 710 711 712 713 714 715 716 717 718 719 720 721 722 723 724 725 726 727 728 729 730 731 732 733 734 735 736
    737 738 739 740 741 742 743 744 745 746 747 748 749 750 751 752 753 754 755 756 757 758 759 760 761 762 763 764 765 766 767 768
    769 770 771 772 773 774 775 776 777 778 779 780 781 782 783 784 785 786 787 788 789 790 791 792 793 794 795 796 797 798 799 800
    801 802 803 804 805 806 807 808 809 810 811 812 813 814 815 816 817 818 819 820 821 822 823 824 825 826 827 828 829 830 831 832
    833 834 835 836 837 838 839 840 841 842 843 844 845 846 847 848 849 850 851 852 853 854 855 856 857 858 859 860 861 862 863 864
    865 866 867 868 869 870 871 872 873 874 875 876 877 878 879 880 881 882 883 884 885 886 887 888 889 890 891 892 893 894 895 896
    897 898 899 900 901 902 903 904 905 906 907 908 909 910 911 912 913 914 915 916 917 918 919 920 921 922 923 924 925 926 927 928
    929 930 931 932 933 934 935 936 937 938 939 940 941 942 943 944 945 946 947 948 949 950 951 952 953 954 955 956 957 958 959 960
    961 962 963 964 965 966 967 968 969 970 971 972 973 974 975 976 977 978 979 980 981 982 983 984 985 986 987 988 989 990 991 992
    0993 0994 0995 0996 0997 0998 0999 1000 1001 1002 1003 1004 1005 1006 1007 1008
    1009 1010 1011 1012 1013 1014 1015 1016 1017 1018 1019 1020 1021 1022 1023 1024
}

unsafe impl<T:Layout<std430>> Layout<std430> for [T] {}
unsafe impl<T:AlignedVec4> Layout<std140> for [T] {}
unsafe impl<T:AlignedVec4> AlignedVec4 for [T] {}

//for uniforms defined with an unnamed struct as a type
macro_rules! impl_tuple_type {
    ($var:ident @first $T0:ident $($T:ident)*) => {$T0::first_element_name($var)};
    ($($T:ident:$t:ident)*) => {

        unsafe impl<$($T:GLSLType),*> GLSLType for ($($T),*) {

            //tuples aren't allowed to be attributes
            type AttributeFormat = UnsupportedFormat;

            unsafe fn load_uniforms(id: GLint, data: &[Self]){
                let mut i = id;
                for ($($t),*) in data {
                    $(
                        $T::load_uniform(i, $t);
                        *(&mut i) = i + $T::uniform_locations() as GLint;
                    )*
                }
            }

            unsafe fn get_uniform(p: GLuint, id:GLint) -> Self {
                let ($(mut $t),*) = ::std::mem::uninitialized::<Self>();
                let mut i = id;
                $(
                    *(&mut $t) = $T::get_uniform(p, i);
                    *(&mut i) = i + $T::uniform_locations() as GLint;
                )*
                ($($t),*)
            }

            #[inline] fn uniform_locations() -> GLuint { 0 $(+ $T::uniform_locations())* }
            #[inline] fn first_element_name(var: String) -> String {impl_tuple_type!(var @first $($T)*)}

        }

    }
}

macro_rules! impl_tuple_layout {

    ({$($T:ident:$t0:ident)*} $Last:ident:$last:ident) => {

        //TODO fix to where a tuple is vec4 aligned if at least one of its members is
        unsafe impl<$($T:Sized+AlignedVec4, )* $Last:?Sized+AlignedVec4> AlignedVec4 for ($($T,)* $Last) {}
        unsafe impl<$($T:Sized+Layout<std140>, )* $Last:?Sized+Layout<std140>> Layout<std140> for ($($T,)* $Last) {}
        unsafe impl<$($T:Sized+Layout<std430>, )* $Last:?Sized+Layout<std430>> Layout<std430> for ($($T,)* $Last) {}

    };
}

impl_tuple!(impl_tuple_type);
impl_tuple!(impl_tuple_layout @with_last);

//
//For specifying which types can be used as data for vertex attributes of the various glsl types
//and what formatting to use
//

macro_rules! impl_attr_data {

    (@Int $prim:ident $value:expr) => {
        impl AttributeData<gl_bool> for $prim { fn format() -> IntFormat { $value }}
        impl AttributeData<int> for $prim { fn format() -> IntFormat { $value }}
        impl AttributeData<uint> for $prim { fn format() -> IntFormat { $value }}
        impl AttributeData<float> for $prim { fn format() -> FloatFormat { FloatFormat::FromInt($value, false) }}
    };

    (@IVec $F:ident $size:tt) => { IVecFormat::IVecN($F::format(), $size) };
    (@Vec $F:ident $size:tt) => { VecFormat::VecN($F::format(), $size) };
    (@DVec $F:ident $size:tt) => { DVecFormat::DVecN($size) };
    (@Mat $F:ident $size:tt) => { [$F::format(); $size] };

    (@$arr:ident $vec1:ident $vec2:ident $vec3:ident $vec4:ident) => {
        impl<F:AttributeData<$vec1>> AttributeData<$vec1> for [F; 1] { fn format() -> <$vec1 as GLSLType>::AttributeFormat { F::format()}}
        impl<F:AttributeData<$vec1>> AttributeData<$vec2> for [F; 2] { fn format() -> <$vec2 as GLSLType>::AttributeFormat { impl_attr_data!(@$arr F 2) } }
        impl<F:AttributeData<$vec1>> AttributeData<$vec3> for [F; 3] { fn format() -> <$vec3 as GLSLType>::AttributeFormat { impl_attr_data!(@$arr F 3) } }
        impl<F:AttributeData<$vec1>> AttributeData<$vec4> for [F; 4] { fn format() -> <$vec4 as GLSLType>::AttributeFormat { impl_attr_data!(@$arr F 4) } }
    };

}

impl_attr_data!(@Int bool IntFormat::UByte);
impl_attr_data!(@Int gl_bool IntFormat::UInt);
impl_attr_data!(@Int i8 IntFormat::Byte);
impl_attr_data!(@Int u8 IntFormat::UByte);
impl_attr_data!(@Int i16 IntFormat::Short);
impl_attr_data!(@Int u16 IntFormat::UShort);
impl_attr_data!(@Int i32 IntFormat::Int);
impl_attr_data!(@Int u32 IntFormat::UInt);

impl AttributeData<float> for f32 { fn format() -> FloatFormat { FloatFormat::Float(FloatType::Float) }}
impl AttributeData<float> for f64 { fn format() -> FloatFormat { FloatFormat::Double }}

impl AttributeData<double> for f64 { fn format() -> DoubleFormat {DoubleFormat}}

impl_attr_data!(@IVec gl_bool bvec2 bvec3 bvec4);
impl_attr_data!(@IVec uint uvec2 uvec3 uvec4);
impl_attr_data!(@IVec int ivec2 ivec3 ivec4);
impl_attr_data!(@Vec float vec2 vec3 vec4);
impl_attr_data!(@Mat vec2 mat2 mat3x2 mat4x2);
impl_attr_data!(@Mat vec3 mat2x3 mat3 mat4x3);
impl_attr_data!(@Mat vec4 mat2x4 mat3x4 mat4);

impl_attr_data!(@DVec double dvec2 dvec3 dvec4);
impl_attr_data!(@Mat dvec2 dmat2 dmat3x2 dmat4x2);
impl_attr_data!(@Mat dvec3 dmat2x3 dmat3 dmat4x3);
impl_attr_data!(@Mat dvec4 dmat2x4 dmat3x4 dmat4);
