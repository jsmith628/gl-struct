use super::*;

glenum! {
    pub enum LogicOp {
        [Clear CLEAR "Clear"],
        [Set SET "Set"],
        [Copy COPY "Copy"],
        [CopyInverted COPY_INVERTED "Copy Inverted"],
        [Noop NOOP "No-op"],
        [Invert INVERT "Invert"],
        [And AND "And"],
        [Nand NAND "Nand"],
        [Or OR "Or"],
        [Nor NOR "Nor"],
        [Xor XOR "Xor"],
        [Equiv EQUIV "Equiv"],
        [AndReverse AND_REVERSE "And Reverse"],
        [AndInverted AND_INVERTED "And Inverted"],
        [OrReverse OR_REVERSE "Or Reverse"],
        [OrInverted OR_INVERTED "Or Inverted"]
    }
}

type TruthTable = [[bool;2];2];

impl LogicOp {

    pub fn from_op<F:Fn(bool,bool)->bool>(op: F) -> Self {
        Self::from_truth_table([
            [op(false,false),op(false,true)],
            [op(true,false),op(true,true)]
        ])
    }

    pub fn into_op(self) -> Box<dyn Fn(bool,bool)->bool> {Box::new(move |r,l| self.op(r,l))}

    pub fn from_truth_table(table: TruthTable) -> Self {

        use self::LogicOp::*;

        match table {
            [[false, false], [false, false]] => Clear,
            [[true, true], [true, true]] => Set,
            [[false, false], [true, true]] => Copy,
            [[true, true], [false, false]] => CopyInverted,
            [[false, true], [false, true]] => Noop,
            [[true, false], [true, false]] => Invert,
            [[false, false], [false, true]] => And,
            [[true, true], [true, false]] => Nand,
            [[false, true], [true, true]] => Or,
            [[true, false], [false, false]] => Nor,
            [[false, true], [true, false]] => Xor,
            [[true, false], [false, true]] => Equiv,
            [[false, false], [true, false]] => AndReverse,
            [[false, true], [false, false]] => AndInverted,
            [[true, false], [true, true]] => OrReverse,
            [[true, true], [false, true]] => OrInverted
        }
    }

    pub fn into_truth_table(self) -> TruthTable {
        [[self.op(false,false),self.op(false,true)],[self.op(true,false),self.op(true,true)]]
    }

    pub fn op_fn<S:FnOnce()->bool, D:FnOnce()->bool>(self, src:S, dst:D) -> bool {
        use self::LogicOp::*;
        match self {
            Clear => false,
            Set => true,
            Copy => src(),
            CopyInverted => !src(),
            Noop => dst(),
            Invert => !dst(),
            And => src() && dst(),
            Nand => !(src() && dst()),
            Or => src() || dst(),
            Nor => !(src() || dst()),
            Xor => src() ^ dst(),
            Equiv => src() == dst(),
            AndReverse => src() && !dst(),
            AndInverted => !src() && dst(),
            OrReverse => src() || !dst(),
            OrInverted => !src() || dst(),
        }
    }

    pub fn op(self, src:bool, dst:bool) -> bool { self.op_fn(||src, ||dst) }
}

macro_rules! impl_ops {
    ($trait:ident::$fun:ident $op:tt $assign:ident::$assign_fun:ident) => {
        impl $trait for LogicOp {
            type Output = Self;
            fn $fun(self, rhs:Self) -> Self {
                [
                    [self.op(false,false) $op rhs.op(false,false), self.op(false,true) $op rhs.op(false,true)],
                    [self.op(true,false) $op rhs.op(true,false), self.op(true,true) $op rhs.op(true,true)]
                ].into()
            }
        }

        impl $assign for LogicOp { fn $assign_fun(&mut self, rhs:Self) { *self = *self $op rhs;} }

        impl $trait<TruthTable> for LogicOp {
            type Output = Self;
            fn $fun(self, rhs:TruthTable) -> Self {
                [
                    [self.op(false,false) $op rhs[0][0], self.op(false,true) $op rhs[0][1]],
                    [self.op(true,false) $op rhs[1][0], self.op(true,true) $op rhs[1][1]]
                ].into()
            }
        }

        impl $assign<TruthTable> for LogicOp { fn $assign_fun(&mut self, rhs:TruthTable) { *self = *self $op rhs;} }
    }
}

impl_ops!(BitAnd::bitand & BitAndAssign::bitand_assign);
impl_ops!(BitOr::bitor | BitOrAssign::bitor_assign);
impl_ops!(BitXor::bitxor ^ BitXorAssign::bitxor_assign);

impl Not for LogicOp {
    type Output = Self;
    fn not(self) -> Self {
        use self::LogicOp::*;
        match self {
            Clear => Set, Set => Clear,
            Copy => CopyInverted, CopyInverted => Copy,
            Noop => Invert, Invert => Noop,
            And => Nand, Nand => And,
            Or => Nor, Nor => Or,
            Xor => Equiv, Equiv => Xor,

            //De Morgan laws
            AndReverse => OrInverted, OrInverted => AndReverse,
            AndInverted => OrReverse, OrReverse => AndInverted,
        }
    }
}


impl<F:Fn(bool,bool)->bool> From<F> for LogicOp { fn from(op:F) -> Self {Self::from_op(op)}}
impl Into<Box<dyn Fn(bool,bool)->bool>> for LogicOp { fn into(self) -> Box<dyn Fn(bool,bool)->bool> {self.into_op()}}

impl From<TruthTable> for LogicOp { fn from(table:TruthTable) -> Self {Self::from_truth_table(table)}}
impl Into<TruthTable> for LogicOp { fn into(self) -> TruthTable {self.into_truth_table()}}
