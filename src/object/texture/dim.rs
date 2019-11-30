use super::*;


pub unsafe trait TexDim: Sized + Copy + Eq + Hash + Debug {
    fn dim() -> usize;
    fn new(width:usize, height:usize, depth:usize) -> Self;
    fn minimized(&self, level: GLuint) -> Self;

    #[inline] fn pixels(&self) -> usize {self.width() * self.height() * self.depth()}
    #[inline] fn max_levels(&self) -> GLuint {
        (0 as GLuint).leading_zeros() - (self.width().max(self.height().max(self.depth()))).leading_zeros()
    }

    #[inline] fn width(&self) -> usize {1}
    #[inline] fn height(&self) -> usize {1}
    #[inline] fn depth(&self) -> usize {1}

}

unsafe impl TexDim for usize {
    #[inline] fn dim() -> usize {1}
    #[inline] fn new(width:usize, _:usize, _:usize) -> Self {width}
    #[inline] fn width(&self) -> usize {*self}
    #[inline] fn minimized(&self, _: GLuint) -> Self { *self }
}

unsafe impl TexDim for [usize;1] {
    #[inline] fn dim() -> usize {1}
    #[inline] fn new(width:usize, _:usize, _:usize) -> Self {[width]}
    #[inline] fn width(&self) -> usize {self[0]}
    #[inline] fn minimized(&self, level: GLuint) -> Self { [(self[0] >> level).max(1)] }
}

unsafe impl TexDim for [usize;2] {
    #[inline] fn dim() -> usize {2}
    #[inline] fn new(width:usize, height:usize, _:usize) -> Self {[width, height]}
    #[inline] fn width(&self) -> usize {self[0]}
    #[inline] fn height(&self) -> usize {self[1]}
    #[inline] fn minimized(&self, level: GLuint) -> Self {
        [(self[0] >> level).max(1), (self[1] >> level).max(1)]
    }
}

unsafe impl TexDim for [usize;3] {
    #[inline] fn dim() -> usize {3}
    #[inline] fn new(width:usize, height:usize, depth:usize) -> Self {[width, height, depth]}
    #[inline] fn width(&self) -> usize {self[0]}
    #[inline] fn height(&self) -> usize {self[1]}
    #[inline] fn depth(&self) -> usize {self[2]}
    #[inline] fn minimized(&self, level: GLuint) -> Self {
        [(self[0] >> level).max(1), (self[1] >> level).max(1), (self[2] >> level).max(1)]
    }
}

unsafe impl TexDim for ([usize;1], usize) {
    #[inline] fn dim() -> usize {2}
    #[inline] fn new(width:usize, height:usize, _:usize) -> Self {([width], height)}
    #[inline] fn minimized(&self, level: GLuint) -> Self {(self.0.minimized(level), self.1)}
    #[inline] fn max_levels(&self) -> GLuint {self.0.max_levels()}

    #[inline] fn width(&self) -> usize {self.0[0]}
    #[inline] fn height(&self) -> usize {self.1}
}

unsafe impl TexDim for ([usize;2], usize) {
    #[inline] fn dim() -> usize {3}
    #[inline] fn new(width:usize, height:usize, depth:usize) -> Self {([width, height], depth)}
    #[inline] fn minimized(&self, level: GLuint) -> Self {(self.0.minimized(level), self.1)}
    #[inline] fn max_levels(&self) -> GLuint {self.0.max_levels()}

    #[inline] fn width(&self) -> usize {self.0[0]}
    #[inline] fn height(&self) -> usize {self.0[1]}
    #[inline] fn depth(&self) -> usize {self.1}
}


pub(super) fn dest_size_check<D:TexDim,F:ClientFormat,I:ImageSrc<F>>(dim:D, p:&I) {
    if D::dim()>=1 && p.width()  < dim.width()  {panic!("Image destination width too small");}
    if D::dim()>=2 && p.height() < dim.height() {panic!("Image destination height too small");}
    if D::dim()>=3 && p.depth()  < dim.depth()  {panic!("Image destination depth too small");}
}

pub(super) fn size_check<D:TexDim,F:ClientFormat,I:ImageSrc<F>>(offset:D, dim:D, p:&I) {
    if D::dim()>=1 && p.width()  != dim.width()  {panic!("Image widths unequal");}
    if D::dim()>=2 && p.height() != dim.height() {panic!("Image heights unequal");}
    if D::dim()>=3 && p.depth()  != dim.depth()  {panic!("Image depths unequal");}
}

pub(super) fn source_size_check<D:TexDim,F:ClientFormat,I:ImageSrc<F>>(offset:D, dim:D, p:&I) {
    if D::dim()>=1 && p.width()+offset.width()   > dim.width()  {panic!("Image source width out of bounds");}
    if D::dim()>=2 && p.height()+offset.height() > dim.height() {panic!("Image source height out of bounds");}
    if D::dim()>=3 && p.depth()+offset.depth()   > dim.depth()  {panic!("Image source depth out of bounds");}
}
