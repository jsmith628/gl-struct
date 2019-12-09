use super::*;

unsafe impl GLSLType for void {
    type AttribFormat = ();
    unsafe fn load_uniforms(_: GLint, _: &[Self]){}
    unsafe fn get_uniform(_: GLuint, _:GLint) -> Self {()}
    fn uniform_locations() -> GLuint { 1 }
}

macro_rules! impl_glsl_type {

    () => {};

    ($ty:ident $set:ident $get:ident [$attr:ident; $num:tt] $($rest:tt)*) => {

        unsafe impl GLSLType for $ty {
            type AttribFormat = [OffsetFormat<$attr>; $num];

            unsafe fn load_uniforms(id: GLint, data: &[Self]){
                $set(id, data.len() as GLint, false as GLboolean, transmute(&data[0][0][0]))
            }

            unsafe fn get_uniform(p: GLuint, id:GLint) -> Self {
                let mut data = MaybeUninit::<Self>::uninit();
                $get(p, id, transmute(data.as_mut_ptr()));
                data.assume_init()
            }
        }

        impl_glsl_type!($($rest)*);

    };

    ($ty:ident $set:ident $get:ident $attr:ident $($rest:tt)*) => {
        unsafe impl GLSLType for $ty {
            type AttribFormat = $attr;

            unsafe fn load_uniforms(id: GLint, data: &[Self]){
                $set(id, data.len() as GLint, transmute(&data[0]))
            }

            unsafe fn get_uniform(p: GLuint, id:GLint) -> Self {
                let mut data = MaybeUninit::<Self>::uninit();
                $get(p, id, transmute(data.as_mut_ptr()));
                data.assume_init()
            }
        }

        impl_glsl_type!($($rest)*);

    }
}

impl_glsl_type! {
    gl_bool Uniform1uiv GetUniformuiv IVecFormat
    bvec2   Uniform2uiv GetUniformuiv IVecFormat
    bvec3   Uniform3uiv GetUniformuiv IVecFormat
    bvec4   Uniform4uiv GetUniformuiv IVecFormat

    uint  Uniform1uiv GetUniformuiv IVecFormat
    uvec2 Uniform2uiv GetUniformuiv IVecFormat
    uvec3 Uniform3uiv GetUniformuiv IVecFormat
    uvec4 Uniform4uiv GetUniformuiv IVecFormat

    int   Uniform1iv GetUniformiv IVecFormat
    ivec2 Uniform2iv GetUniformiv IVecFormat
    ivec3 Uniform3iv GetUniformiv IVecFormat
    ivec4 Uniform4iv GetUniformiv IVecFormat

    float Uniform1fv GetUniformfv VecFormat
    vec2  Uniform2fv GetUniformfv VecFormat
    vec3  Uniform3fv GetUniformfv VecFormat
    vec4  Uniform4fv GetUniformfv VecFormat

    mat2x2 UniformMatrix2fv   GetUniformfv [VecFormat; 2]
    mat2x3 UniformMatrix2x3fv GetUniformfv [VecFormat; 2]
    mat2x4 UniformMatrix2x4fv GetUniformfv [VecFormat; 2]
    mat3x2 UniformMatrix3x2fv GetUniformfv [VecFormat; 3]
    mat3x3 UniformMatrix3fv   GetUniformfv [VecFormat; 3]
    mat3x4 UniformMatrix3x4fv GetUniformfv [VecFormat; 3]
    mat4x2 UniformMatrix4x2fv GetUniformfv [VecFormat; 4]
    mat4x3 UniformMatrix4x3fv GetUniformfv [VecFormat; 4]
    mat4x4 UniformMatrix4fv   GetUniformfv [VecFormat; 4]

    double Uniform1dv GetUniformdv DVecFormat
    dvec2  Uniform2dv GetUniformdv DVecFormat
    dvec3  Uniform3dv GetUniformdv DVecFormat
    dvec4  Uniform4dv GetUniformdv DVecFormat

    dmat2x2 UniformMatrix2dv   GetUniformdv [DVecFormat; 2]
    dmat2x3 UniformMatrix2x3dv GetUniformdv [DVecFormat; 2]
    dmat2x4 UniformMatrix2x4dv GetUniformdv [DVecFormat; 2]
    dmat3x2 UniformMatrix3x2dv GetUniformdv [DVecFormat; 3]
    dmat3x3 UniformMatrix3dv   GetUniformdv [DVecFormat; 3]
    dmat3x4 UniformMatrix3x4dv GetUniformdv [DVecFormat; 3]
    dmat4x2 UniformMatrix4x2dv GetUniformdv [DVecFormat; 4]
    dmat4x3 UniformMatrix4x3dv GetUniformdv [DVecFormat; 4]
    dmat4x4 UniformMatrix4dv  GetUniformdv [DVecFormat; 4]
}

macro_rules! impl_tuple_type {
    ($($A:ident:$a:ident)*) => {
        unsafe impl<$($A:GLSLType),*> GLSLType for ($($A,)*) {
            type AttribFormat = !;

            #[allow(unused_assignments)]
            unsafe fn load_uniforms(id: GLint, data: &[Self]){
                let mut i = id;
                for ($($a,)*) in data.iter() {
                    $(
                        $A::load_uniform(i, $a);
                        i += $A::uniform_locations() as GLint;
                    )*
                }
            }

            #[allow(unused_assignments)]
            unsafe fn get_uniform(p: GLuint, id:GLint) -> Self {
                let mut i = id;
                (
                    $({
                        let val = $A::get_uniform(p, i);
                        i += $A::uniform_locations() as GLint;
                        val
                    },)*
                )
            }

            #[inline] fn uniform_locations() -> GLuint { 0 $( + $A::uniform_locations())*}
        }
    }
}

impl_tuple!(impl_tuple_type);

macro_rules! impl_array_type {

    (@attrib true $ty:ty) => {$ty};
    (@attrib false $ty:ty) => {!};

    ($attrib_support:tt $($num:tt)*) => {
        $(
            unsafe impl<T:GLSLType> GLSLType for [T; $num] {
                type AttribFormat = impl_array_type!(
                    @attrib $attrib_support [OffsetFormat<T::AttribFormat>; $num]
                );

                unsafe fn load_uniforms(id: GLint, data: &[Self]){
                    let flattened = from_raw_parts(&data[0][0] as *const T, data.len() * $num);
                    T::load_uniforms(id, flattened);
                }

                unsafe fn get_uniform(p: GLuint, id:GLint) -> Self {
                    let mut data = MaybeUninit::<Self>::uninit();
                    for i in 0..$num {
                        data.get_mut()[i] = T::get_uniform(p, id + i as GLint);
                    }
                    data.assume_init()
                }

                #[inline] fn uniform_locations() -> GLuint { T::uniform_locations() * $num }
                // #[inline] fn first_element_name(var: String) -> String { T::first_element_name(var + "[0]") }

            }

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
