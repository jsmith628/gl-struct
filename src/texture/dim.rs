use super::*;

pub unsafe trait TexDim:Copy {
    fn dim() -> usize;
    fn minimized(&self, level: GLuint) -> Self;

    #[inline] fn pixels(&self) -> usize {self.width() * self.height() * self.depth()}
    #[inline] fn max_levels(&self) -> GLuint {
        (0 as GLuint).leading_zeros() - (self.width().max(self.height().max(self.depth()))).leading_zeros()
    }

    #[inline] fn width(&self) -> usize {1}
    #[inline] fn height(&self) -> usize {1}
    #[inline] fn depth(&self) -> usize {1}

}

unsafe impl TexDim for [usize;1] {
    #[inline] fn dim() -> usize {1}
    #[inline] fn width(&self) -> usize {self[0]}
    #[inline] fn minimized(&self, level: GLuint) -> Self { [(self[0] >> level).max(1)] }
}

unsafe impl TexDim for [usize;2] {
    #[inline] fn dim() -> usize {2}
    #[inline] fn width(&self) -> usize {self[0]}
    #[inline] fn height(&self) -> usize {self[1]}
    #[inline] fn minimized(&self, level: GLuint) -> Self {
        [(self[0] >> level).max(1), (self[1] >> level).max(1)]
    }
}

unsafe impl TexDim for [usize;3] {
    #[inline] fn dim() -> usize {3}
    #[inline] fn width(&self) -> usize {self[0]}
    #[inline] fn height(&self) -> usize {self[1]}
    #[inline] fn depth(&self) -> usize {self[2]}
    #[inline] fn minimized(&self, level: GLuint) -> Self {
        [(self[0] >> level).max(1), (self[1] >> level).max(1), (self[2] >> level).max(1)]
    }
}

unsafe impl TexDim for ([usize;1], usize) {
    #[inline] fn dim() -> usize {2}
    #[inline] fn minimized(&self, level: GLuint) -> Self {(self.0.minimized(level), self.1)}
    #[inline] fn max_levels(&self) -> GLuint {self.0.max_levels()}

    #[inline] fn width(&self) -> usize {self.0[0]}
    #[inline] fn height(&self) -> usize {self.1}
}

unsafe impl TexDim for ([usize;2], usize) {
    #[inline] fn dim() -> usize {3}
    #[inline] fn minimized(&self, level: GLuint) -> Self {(self.0.minimized(level), self.1)}
    #[inline] fn max_levels(&self) -> GLuint {self.0.max_levels()}

    #[inline] fn width(&self) -> usize {self.0[0]}
    #[inline] fn height(&self) -> usize {self.0[1]}
    #[inline] fn depth(&self) -> usize {self.1}
}
