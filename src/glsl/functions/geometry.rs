use super::*;

pub trait CrossProduct: GenFloatType {
    fn cross(self, rhs:Self) -> Self;
}

macro_rules! impl_cross {
    ($ty:ident) => {
        impl CrossProduct for $ty {
            fn cross(self, rhs:Self) -> Self {
                $ty {
                    value: [
                        self[1]*rhs[2] - rhs[1]*self[2],
                        self[2]*rhs[0] - rhs[2]*self[0],
                        self[0]*rhs[1] - rhs[0]*self[1]
                    ]
                }
            }
        }
    }
}

impl_cross!(vec3);
impl_cross!(dvec3);

pub fn length<V:GenFloatType>(v:V) -> V::Component { dot(v,v).sqrt() }

pub fn distance<V:GenFloatType>(p0:V, p1:V) -> V::Component { length(p0-p1) }

pub fn dot<V:GenFloatType>(x:V, y:V) -> V::Component {
    (0..x.length()).fold(num_traits::Zero::zero(), |d,i| d+ *x.coord(i)**y.coord(i))
}

pub fn cross<V:CrossProduct>(x:V, y:V) -> V { x.cross(y) }

pub fn normalize<V:GenFloatType>(x:V) -> V { x / length(x) }

#[allow(non_snake_case)]
pub fn faceForward<V:GenFloatType>(N:V, I:V, Nref:V) -> V {
    if dot(Nref,I) < Zero::zero() { N } else { -N }
}

#[allow(non_snake_case)]
pub fn reflect<V:GenFloatType>(I:V, N:V) -> V {
    I - ((V::Component::one()+One::one())*dot(N,I))*N
}

#[allow(non_snake_case)]
pub fn refract<V:GenFloatType>(I:V, N:V, eta: V::Component) -> V {
    let k:V::Component = V::Component::one() - eta*eta*(V::Component::one() - dot(N,I)*dot(N,I));
    if k < Zero::zero() {
        Zero::zero()
    } else {
        eta*I - (eta*dot(N,I) + k.sqrt())*N
    }
}
