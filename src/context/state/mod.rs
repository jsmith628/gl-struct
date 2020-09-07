use super::*;

use std::mem::*;
use std::convert::TryInto;
use std::ops::*;
use std::ffi::*;
use std::os::raw::c_char;

use self::logic_op::*;
use self::debug_output::*;

mod logic_op;
mod debug_output;

glenum! {

    pub enum BlendEquation {
        [FuncAdd FUNC_ADD "Func Add"],
        [FuncSubtract FUNC_SUBTRACT "Func Subtract"],
        [FuncReverseSubtract FUNC_REVERSE_SUBTRACT "Func Reverse Subtract"],
        [Min MIN "Min"],
        [Max MAX "Max"]
    }

    pub enum BlendFunc {
        [Zero ZERO "Zero"],
        [One ONE "One"],
        [SRCColor SRC_COLOR "SRC Color"],
        [OneMinusSRCColor ONE_MINUS_SRC_COLOR "One Minus SRC Color"],
        [DSTColor DST_COLOR "DST Color"],
        [OneMinusDSTColor ONE_MINUS_DST_COLOR "One Minus DST Color"],
        [SRCAlpha SRC_ALPHA "SRC Alpha"],
        [OneMinusSRCAlpha ONE_MINUS_SRC_ALPHA "One Minus SRC Alpha"],
        [DSTAlpha DST_ALPHA "DST Alpha"],
        [OneMinusDSTAlpha ONE_MINUS_DST_ALPHA "One Minus DST Alpha"],
        [ConstantColor CONSTANT_COLOR "Constant Color"],
        [OneMinusConstantColor ONE_MINUS_CONSTANT_COLOR "One Minus Constant Color"],
        [ConstantAlpha CONSTANT_ALPHA "Constant Alpha"],
        [OneMinusConstantAlpha ONE_MINUS_CONSTANT_ALPHA "One Minus Constant Alpha"],
        [SRCAlphaSaturate SRC_ALPHA_SATURATE "SRC Alpha Saturate"],
        [SRC1Color SRC1_COLOR "SRC1 Color"],
        [OneMinusSRC1Color ONE_MINUS_SRC1_COLOR "One Minus SRC1 Color"],
        [SRC1Alpha SRC1_ALPHA "SRC1 Alpha"],
        [OneMinusSRC1Alpha ONE_MINUS_SRC1_ALPHA "One Minus SRC1 Alpha"]
    }

    pub enum ClampColorTarget {
        [ClampReadColor CLAMP_READ_COLOR "Clamp Read Color"]
    }

    pub enum CoordOrigin {
        [LowerLeft LOWER_LEFT "Lower Left"],
        [UpperLeft UPPER_LEFT "Upper Left"]
    }

    pub enum ClipDepthMode {
        [NegativeOneToOne NEGATIVE_ONE_TO_ONE "Negative one to one"],
        [ZeroToOne ZERO_TO_ONE "Zero to one"]
    }

    pub enum PolygonFace {
        [Front FRONT "Front"],
        [Back BACK "Front"],
        [FrontAndBack FRONT_AND_BACK "Front and Back"]
    }

    pub enum VertexWinding {
        [CCW CCW "Counter-Clockwise"],
        [CW CW "Clockwise"]
    }

    pub enum CompareFunc {
        [Never NEVER "Never"],
        [Less LESS "Less"],
        [Equal EQUAL "Equal"],
        [Lequal LEQUAL "Lequal"],
        [Greater GREATER "Greater"],
        [Notequal NOTEQUAL "Notequal"],
        [Gequal GEQUAL "Gequal"],
        [Always ALWAYS "Always"]
    }

    pub enum Error {
        [NoError NO_ERROR "No Error"],
        [InvalidEnum INVALID_ENUM "Invalid Enum"],
        [InvalidValue INVALID_VALUE "Invalid Value"],
        [InvalidOperation INVALID_OPERATION "Invalid Operation"],
        [InvalidFramebufferOperation INVALID_FRAMEBUFFER_OPERATION "Invalid Framebuffer Operation"],
        [OutOfMemory OUT_OF_MEMORY "Out of Memory"],
        [StackUnderflow STACK_UNDERFLOW "Stack Underflow"],
        [StackOverflow STACK_OVERFLOW "Stack Overflow"]
    }

    pub enum Hint {
        [Fastest FASTEST "Fastest"],
        [Nicest NICEST "Nicest"],
        [DontCare DONT_CARE "Dont Care"]
    }

    pub enum PolygonMode {
        [Point POINT "Point"],
        [Line LINE "Line"],
        [Fill FILL "Fill"]
    }

    pub enum StencilOp {
        [Keep KEEP "Keep"],
        [Zero ZERO "Zero"],
        [Replace REPLACE "Replace"],
        [Incr INCR "Increase"],
        [IncrWrap INCR_WRAP "Wrapping Increase"],
        [Decr DECR "Decrease"],
        [DecrWrap DECR_WRAP "Wrapping Decrease"],
        [Invert INVERT "Invert"]
    }

}

pub struct GLState<V:GLVersion> {
    version: std::marker::PhantomData<Box<V>>,
    debug_callback: DebugCallback
}

impl<V:GLVersion> Drop for GLState<V> {
    fn drop(&mut self) {

        trait InnerDrop { fn inner(&mut self); }
        impl<V:GLVersion> InnerDrop for GLState<V> { default fn inner(&mut self) {}}
        impl<V:Supports<GL43>+Supports<GL20>> InnerDrop for GLState<V> {
            fn inner(&mut self) {
                //make sure the debug callback isn't called after the function is dropped
                self.debug_message_callback_null();
            }
        }

        self.inner()

    }
}

//TODO: add the compatibility profile parameters

impl<V:Supports<GL20>> GLState<V> {

    unsafe fn get_boolean(&self, pname: GLenum) -> bool {
        let mut dest = MaybeUninit::uninit();
        gl::GetBooleanv(pname, dest.as_mut_ptr());
        dest.assume_init() != 0
    }

    unsafe fn get_unsigned_integer(&self, pname: GLenum) -> GLuint { self.get_integer(pname) as GLuint }
    unsafe fn get_glenum<T:GLEnum>(&self, pname: GLenum) -> T {self.get_unsigned_integer(pname).try_into().unwrap()}
    unsafe fn get_integer(&self, pname: GLenum) -> GLint {
        let mut dest = MaybeUninit::uninit();
        gl::GetIntegerv(pname, dest.as_mut_ptr());
        dest.assume_init()
    }

    unsafe fn get_float(&self, pname: GLenum) -> GLfloat {
        let mut dest = MaybeUninit::uninit();
        gl::GetFloatv(pname, dest.as_mut_ptr());
        dest.assume_init()
    }

    unsafe fn get_float_range(&self, pname:GLenum) -> RangeInclusive<GLfloat> {
        let mut dest = MaybeUninit::<[GLfloat;2]>::uninit();
        gl::GetFloatv(pname, &mut dest.assume_init_mut()[0] as *mut GLfloat);
        let dest = dest.assume_init();
        dest[0]..=dest[1]
    }

    #[allow(dead_code)]
    unsafe fn get_double(&self, pname: GLenum) -> GLdouble {
        let mut dest = MaybeUninit::uninit();
        gl::GetDoublev(pname, dest.as_mut_ptr());
        dest.assume_init()
    }

    unsafe fn get_string(&self, pname: GLenum) -> &CStr {
        CStr::from_ptr(gl::GetString(pname) as *const c_char)
    }

    //
    //glGetString
    //

    pub fn get_vendor(&self) -> &CStr { unsafe {self.get_string(gl::VENDOR)} }
    pub fn get_renderer(&self) -> &CStr { unsafe {self.get_string(gl::RENDERER)} }
    pub fn get_version(&self) -> &CStr { unsafe {self.get_string(gl::VERSION)} }
    pub fn get_shading_language_version(&self) -> &CStr { unsafe {self.get_string(gl::SHADING_LANGUAGE_VERSION)} }

    //TODO fill in the important glGet parameters

    //
    //Blending
    //

    pub fn blend_color(&mut self, red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat) {
        unsafe { gl::BlendColor(red, green, blue, alpha); }
    }

    pub fn get_blend_color(&self) -> [GLfloat;4] {
        unsafe {
            let mut dest = MaybeUninit::<[GLfloat;4]>::uninit();
            gl::GetFloatv(gl::BLEND_COLOR, dest.as_mut_ptr() as *mut GLfloat);
            dest.assume_init()
        }
    }

    //
    //Culling
    //

    pub fn is_cull_face_enabled(&self) -> bool { unsafe {gl::IsEnabled(gl::CULL_FACE)!= 0} }
    pub fn enable_cull_face(&mut self) { unsafe {gl::Enable(gl::CULL_FACE)} }
    pub fn disable_cull_face(&mut self) { unsafe {gl::Disable(gl::CULL_FACE)} }

    pub fn get_cull_face_mode(&self) -> PolygonFace { unsafe {self.get_glenum(gl::CULL_FACE_MODE)} }
    pub fn cull_face(&mut self, mode: PolygonFace) { unsafe {gl::CullFace(mode as GLenum)} }

    //
    //Depth Test
    //

    pub fn is_depth_test_enabled(&self) -> bool { unsafe {gl::IsEnabled(gl::DEPTH_TEST)!= 0} }
    pub fn enable_depth_test(&mut self) { unsafe {gl::Enable(gl::DEPTH_TEST)} }
    pub fn disable_depth_test(&mut self) { unsafe {gl::Disable(gl::DEPTH_TEST)} }

    pub fn get_depth_func(&self) -> CompareFunc { unsafe {self.get_glenum(gl::DEPTH_FUNC)} }
    pub fn depth_func(&mut self, func: CompareFunc) { unsafe {gl::DepthFunc(func as GLenum)} }

    pub fn depth_mask(&mut self, flag: bool) {
        unsafe {gl::DepthMask(flag as GLboolean)}
    }

    //
    //Dithering
    //

    pub fn is_dither_enabled(&self) -> bool { unsafe {gl::IsEnabled(gl::DITHER)!= 0} }
    pub fn enable_dither(&mut self) { unsafe {gl::Enable(gl::DITHER)} }
    pub fn disable_dither(&mut self) { unsafe {gl::Disable(gl::DITHER)} }

    //
    //Front Face
    //

    pub fn get_front_face(&self) -> CompareFunc { unsafe {self.get_glenum(gl::FRONT_FACE)} }
    pub fn front_face(&mut self, mode: VertexWinding) { unsafe {gl::FrontFace(mode as GLenum)} }

    //
    //Get Error
    //

    pub fn get_error(&mut self) -> Error { unsafe {gl::GetError().try_into().unwrap()} }

    //
    //Performance hints
    //

    // pub enum HintTarget {
    //     [PolygonSmoothHint POLYGON_SMOOTH_HINT "Polygon Smooth Hint"],
    //     [TextureCompressionHint TEXTURE_COMPRESSION_HINT "Texture Compression Hint"],
    //     [FragmentShaderDerivativeHint FRAGMENT_SHADER_DERIVATIVE_HINT "Fragment Shader Derivative Hint"]
    // }

    //
    //Smooth lines
    //

    pub fn is_line_smooth_enabled(&self) -> bool { unsafe {gl::IsEnabled(gl::LINE_SMOOTH)!= 0} }
    pub fn enable_line_smooth(&mut self) { unsafe {gl::Enable(gl::LINE_SMOOTH)} }
    pub fn disable_line_smooth(&mut self) { unsafe {gl::Disable(gl::LINE_SMOOTH)} }

    pub fn line_smooth_hint(&mut self, hint:Hint) { unsafe {gl::Hint(gl::LINE_SMOOTH_HINT, hint as GLenum)} }

    pub fn get_line_width(&self) -> GLfloat { unsafe {self.get_float(gl::LINE_WIDTH)} }
    pub fn get_smooth_line_width_range(&self) -> RangeInclusive<GLfloat> { unsafe {self.get_float_range(gl::SMOOTH_LINE_WIDTH_RANGE)} }
    pub fn get_aliased_line_width_range(&self) -> RangeInclusive<GLfloat> { unsafe {self.get_float_range(gl::ALIASED_LINE_WIDTH_RANGE)} }
    pub fn get_smooth_line_width_granularity(&self) -> GLfloat { unsafe {self.get_float(gl::SMOOTH_LINE_WIDTH_GRANULARITY)} }

    pub fn line_width(&mut self, width: GLfloat) { unsafe {gl::LineWidth(width)} }

    //
    //Color Logic Op
    //

    pub fn is_color_logic_op_enabled(&self) -> bool { unsafe {gl::IsEnabled(gl::COLOR_LOGIC_OP)!= 0} }
    pub fn enable_color_logic_op(&mut self) { unsafe {gl::Enable(gl::COLOR_LOGIC_OP)} }
    pub fn disable_color_logic_op(&mut self) { unsafe {gl::Disable(gl::COLOR_LOGIC_OP)} }

    pub fn get_logic_op_mode(&self) -> LogicOp { unsafe {self.get_glenum(gl::LOGIC_OP_MODE)}  }
    pub fn logic_op(&mut self, opcode: LogicOp) { unsafe {gl::LogicOp(opcode as GLenum)} }

    //
    //Point parameters
    //

    pub fn get_point_fade_threshold_size(&self) -> GLfloat { unsafe {self.get_float(gl::POINT_FADE_THRESHOLD_SIZE)} }
    pub fn point_fade_threshold_size(&mut self, param: GLfloat) {
        unsafe { gl::PointParameterf(gl::POINT_FADE_THRESHOLD_SIZE, param) }
    }

    pub fn get_point_sprite_coord_origin(&self) -> CoordOrigin { unsafe {self.get_glenum(gl::POINT_SPRITE_COORD_ORIGIN)} }
    pub fn point_sprite_coord_origin(&mut self, param: CoordOrigin) {
        unsafe { gl::PointParameteri(gl::POINT_SPRITE_COORD_ORIGIN, param as GLint) }
    }

    //
    //Point Size
    //

    pub fn is_program_point_size_enabled(&self) -> bool { unsafe {gl::IsEnabled(gl::PROGRAM_POINT_SIZE)!= 0} }
    pub fn enable_program_point_size(&mut self) { unsafe {gl::Enable(gl::PROGRAM_POINT_SIZE)} }
    pub fn disable_program_point_size(&mut self) { unsafe {gl::Disable(gl::PROGRAM_POINT_SIZE)} }

    pub fn get_point_size(&self) -> GLfloat { unsafe {self.get_float(gl::POINT_SIZE)} }
    pub fn get_point_size_range(&self) -> RangeInclusive<GLfloat> { unsafe {self.get_float_range(gl::POINT_SIZE_RANGE)} }
    pub fn get_point_size_granularity(&self) -> GLfloat { unsafe {self.get_float(gl::POINT_SIZE_GRANULARITY)} }

    pub fn point_size(&mut self, size: GLfloat) { unsafe {gl::PointSize(size)} }

    //
    //Polygon fill mode
    //

    pub fn get_polygon_mode(&self) -> PolygonMode { unsafe {self.get_glenum(gl::POLYGON_MODE)} }
    pub fn polygon_mode(&mut self, mode: PolygonMode) {
        unsafe { gl::PolygonMode(gl::FRONT_AND_BACK, mode as GLenum) }
    }

    //
    //Polygon offset
    //

    pub fn is_polygon_offset_fill_enabled(&self) -> bool { unsafe {gl::IsEnabled(gl::POLYGON_OFFSET_FILL)!= 0} }
    pub fn enable_polygon_offset_fill_size(&mut self) { unsafe {gl::Enable(gl::POLYGON_OFFSET_FILL)} }
    pub fn disable_polygon_offset_fill_size(&mut self) { unsafe {gl::Disable(gl::POLYGON_OFFSET_FILL)} }

    pub fn is_polygon_offset_line_enabled(&self) -> bool { unsafe {gl::IsEnabled(gl::POLYGON_OFFSET_LINE)!= 0} }
    pub fn enable_polygon_offset_line_size(&mut self) { unsafe {gl::Enable(gl::POLYGON_OFFSET_LINE)} }
    pub fn disable_polygon_offset_line_size(&mut self) { unsafe {gl::Disable(gl::POLYGON_OFFSET_LINE)} }

    pub fn is_polygon_offset_point_enabled(&self) -> bool { unsafe {gl::IsEnabled(gl::POLYGON_OFFSET_POINT)!= 0} }
    pub fn enable_polygon_offset_point_size(&mut self) { unsafe {gl::Enable(gl::POLYGON_OFFSET_POINT)} }
    pub fn disable_polygon_offset_point_size(&mut self) { unsafe {gl::Disable(gl::POLYGON_OFFSET_POINT)} }

    pub fn get_polygon_offset_factor(&self) -> GLfloat { unsafe {self.get_float(gl::POLYGON_OFFSET_FACTOR)} }
    pub fn get_polygon_offset_units(&self) -> GLfloat { unsafe {self.get_float(gl::POLYGON_OFFSET_UNITS)} }

    pub fn polygon_offset(&mut self, factor: GLfloat, units: GLfloat) { unsafe {gl::PolygonOffset(factor, units)} }


    //
    //Multisampling
    //

    pub fn is_multisample_enabled(&self) -> bool { unsafe {gl::IsEnabled(gl::MULTISAMPLE)!= 0} }
    pub fn enable_multisample(&mut self) { unsafe {gl::Enable(gl::MULTISAMPLE)} }
    pub fn disable_multisample(&mut self) { unsafe {gl::Disable(gl::MULTISAMPLE)} }

    pub fn is_sample_alpha_to_coverage_enabled(&self) -> bool { unsafe {gl::IsEnabled(gl::SAMPLE_ALPHA_TO_COVERAGE)!= 0} }
    pub fn enable_sample_alpha_to_coverage(&mut self) { unsafe {gl::Enable(gl::SAMPLE_ALPHA_TO_COVERAGE)} }
    pub fn disable_sample_alpha_to_coverage(&mut self) { unsafe {gl::Disable(gl::SAMPLE_ALPHA_TO_COVERAGE)} }

    pub fn is_sample_alpha_to_one_enabled(&self) -> bool { unsafe {gl::IsEnabled(gl::SAMPLE_ALPHA_TO_ONE)!= 0} }
    pub fn enable_sample_alpha_to_one(&mut self) { unsafe {gl::Enable(gl::SAMPLE_ALPHA_TO_ONE)} }
    pub fn disable_sample_alpha_to_one(&mut self) { unsafe {gl::Disable(gl::SAMPLE_ALPHA_TO_ONE)} }

    pub fn is_sample_coverage_enabled(&self) -> bool { unsafe {gl::IsEnabled(gl::SAMPLE_COVERAGE)!= 0} }
    pub fn enable_sample_coverage(&mut self) { unsafe {gl::Enable(gl::SAMPLE_COVERAGE)} }
    pub fn disable_sample_coverage(&mut self) { unsafe {gl::Disable(gl::SAMPLE_COVERAGE)} }

    pub fn get_sample_coverage_value(&self) -> GLfloat {unsafe {self.get_float(gl::SAMPLE_COVERAGE_VALUE)} }
    pub fn get_sample_coverage_invert(&self) -> bool {unsafe {self.get_boolean(gl::SAMPLE_COVERAGE_INVERT)} }
    pub fn sample_coverage(&mut self, value: GLfloat, invert: bool) {
        unsafe { gl::SampleCoverage(value, invert as GLboolean) }
    }

    //
    //Stencil test
    //

    pub fn is_stencil_test_enabled(&self) -> bool { unsafe {gl::IsEnabled(gl::STENCIL_TEST)!= 0} }
    pub fn enable_stencil_test(&mut self) { unsafe {gl::Enable(gl::STENCIL_TEST)} }
    pub fn disable_stencil_test(&mut self) { unsafe {gl::Disable(gl::STENCIL_TEST)} }

    pub fn get_stencil_func(&self) -> CompareFunc { unsafe {self.get_glenum(gl::STENCIL_FUNC)} }
    pub fn get_stencil_value_mask(&self) -> GLuint { unsafe {self.get_unsigned_integer(gl::STENCIL_VALUE_MASK)} }
    pub fn get_stencil_ref(&self) -> GLint { unsafe {self.get_integer(gl::STENCIL_REF)} }

    pub fn get_stencil_back_func(&self) -> CompareFunc { unsafe {self.get_glenum(gl::STENCIL_BACK_FUNC)} }
    pub fn get_stencil_back_value_mask(&self) -> GLuint { unsafe {self.get_unsigned_integer(gl::STENCIL_BACK_VALUE_MASK)} }
    pub fn get_stencil_back_ref(&self) -> GLint { unsafe {self.get_integer(gl::STENCIL_BACK_REF)} }

    pub fn stencil_func(&mut self, func:CompareFunc, ref_value:GLint, mask:GLuint) {
        unsafe { gl::StencilFunc(func as GLenum, ref_value, mask) }
    }

    pub fn stencil_func_separate(&mut self, face:PolygonFace, func:CompareFunc, ref_value:GLint, mask:GLuint) {
        unsafe { gl::StencilFuncSeparate(face as GLenum, func as GLenum, ref_value, mask) }
    }

    pub fn get_stencil_writemask(&self) -> GLuint { unsafe {self.get_unsigned_integer(gl::STENCIL_WRITEMASK)} }
    pub fn get_stencil_back_writemask(&self) -> GLuint { unsafe {self.get_unsigned_integer(gl::STENCIL_BACK_WRITEMASK)} }

    pub fn stencil_mask(&mut self, mask:GLuint) { unsafe {gl::StencilMask(mask)} }
    pub fn stencil_mask_separate(&mut self, face:PolygonFace, mask:GLuint) { unsafe {gl::StencilMaskSeparate(face as GLenum, mask)} }

    pub fn get_stencil_fail(&self) -> StencilOp { unsafe {self.get_glenum(gl::STENCIL_FAIL)} }
    pub fn get_stencil_pass_depth_pass(&self) -> StencilOp { unsafe {self.get_glenum(gl::STENCIL_PASS_DEPTH_PASS)} }
    pub fn get_stencil_pass_depth_fail(&self) -> StencilOp { unsafe {self.get_glenum(gl::STENCIL_PASS_DEPTH_FAIL)} }

    pub fn get_stencil_back_fail(&self) -> StencilOp { unsafe {self.get_glenum(gl::STENCIL_BACK_FAIL)} }
    pub fn get_stencil_back_pass_depth_pass(&self) -> StencilOp { unsafe {self.get_glenum(gl::STENCIL_BACK_PASS_DEPTH_PASS)} }
    pub fn get_stencil_back_pass_depth_fail(&self) -> StencilOp { unsafe {self.get_glenum(gl::STENCIL_BACK_PASS_DEPTH_FAIL)} }

    pub fn stencil_op(&mut self, sfail:StencilOp, dpfail:StencilOp, dppass:StencilOp) {
        unsafe {gl::StencilOp(sfail as GLenum, dpfail as GLenum, dppass as GLenum)}
    }

    pub fn stencil_op_separate(&mut self, face:PolygonFace, sfail:StencilOp, dpfail:StencilOp, dppass:StencilOp) {
        unsafe {gl::StencilOpSeparate(face as GLenum, sfail as GLenum, dpfail as GLenum, dppass as GLenum)}
    }


}

impl<V:Supports<GL30>> GLState<V> {
    //
    //Seamless Cubemaps
    //

    pub fn is_texture_cube_map_seamless_enabled(&self) -> bool { unsafe {gl::IsEnabled(gl::TEXTURE_CUBE_MAP_SEAMLESS)!= 0} }
    pub fn enable_texture_cube_map_seamless(&mut self) { unsafe {gl::Enable(gl::TEXTURE_CUBE_MAP_SEAMLESS)} }
    pub fn disable_texture_cube_map_seamless(&mut self) { unsafe {gl::Disable(gl::TEXTURE_CUBE_MAP_SEAMLESS)} }

}

impl<V:Supports<GL32>> GLState<V> {

    #[allow(dead_code)]
    unsafe fn get_integer64(&self, pname: GLenum) -> GLint64 {
        let mut dest = MaybeUninit::uninit();
        gl::GetInteger64v(pname, dest.as_mut_ptr());
        dest.assume_init()
    }

    #[allow(dead_code)]
    unsafe fn get_unsinged_integer64(&self, pname: GLenum) -> GLuint64 { self.get_integer64(pname) as GLuint64 }
}

impl<V:Supports<GL45>> GLState<V> {
    pub fn clip_control(&mut self, origin: CoordOrigin, depth: ClipDepthMode) {
        unsafe { gl::ClipControl(origin as GLenum, depth as GLenum) };
    }
}
