use super::*;

pub trait MultisampleFormat {
    const SAMPLES: GLuint;
    const FIXED: bool;
}

pub unsafe trait RenderbufferMSFormat: MultisampleFormat {}

macro_rules! ms {

    (@rb $ms:ident true) => {};
    (@rb $ms:ident false) => { unsafe impl RenderbufferMSFormat for $ms {} };

    ($($ms:ident = ($samples:literal, $fixed:ident);)*) => {
        $(
            pub struct $ms;

            impl MultisampleFormat for $ms {
                const SAMPLES: GLuint = $samples;
                const FIXED: bool = $fixed;
            }

            ms!(@rb $ms $fixed);

        )*
    }
}

ms! {

    MS0  = (0,  false);
    MS1  = (1,  false);
    MS2  = (2,  false);
    MS3  = (3,  false);
    MS4  = (4,  false);
    MS5  = (5,  false);
    MS6  = (6,  false);
    MS7  = (7,  false);
    MS8  = (8,  false);
    MS9  = (9,  false);
    MS10 = (10, false);
    MS11 = (11, false);
    MS12 = (12, false);
    MS13 = (13, false);
    MS14 = (14, false);
    MS15 = (15, false);
    MS16 = (16, false);
    MS17 = (17, false);
    MS18 = (18, false);
    MS19 = (19, false);
    MS20 = (20, false);
    MS21 = (21, false);
    MS22 = (22, false);
    MS23 = (23, false);
    MS24 = (24, false);
    MS25 = (25, false);
    MS26 = (26, false);
    MS27 = (27, false);
    MS28 = (28, false);
    MS29 = (29, false);
    MS30 = (30, false);
    MS31 = (31, false);
    MS32 = (32, false);

    MS1Fixed  = (1,  true);
    MS2Fixed  = (2,  true);
    MS3Fixed  = (3,  true);
    MS4Fixed  = (4,  true);
    MS5Fixed  = (5,  true);
    MS6Fixed  = (6,  true);
    MS7Fixed  = (7,  true);
    MS8Fixed  = (8,  true);
    MS9Fixed  = (9,  true);
    MS10Fixed = (10, true);
    MS11Fixed = (11, true);
    MS12Fixed = (12, true);
    MS13Fixed = (13, true);
    MS14Fixed = (14, true);
    MS15Fixed = (15, true);
    MS16Fixed = (16, true);
    MS17Fixed = (17, true);
    MS18Fixed = (18, true);
    MS19Fixed = (19, true);
    MS20Fixed = (20, true);
    MS21Fixed = (21, true);
    MS22Fixed = (22, true);
    MS23Fixed = (23, true);
    MS24Fixed = (24, true);
    MS25Fixed = (25, true);
    MS26Fixed = (26, true);
    MS27Fixed = (27, true);
    MS28Fixed = (28, true);
    MS29Fixed = (29, true);
    MS30Fixed = (30, true);
    MS31Fixed = (31, true);
    MS32Fixed = (32, true);

}
