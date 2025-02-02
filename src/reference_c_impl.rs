use std::marker::PhantomData;

struct OffsetT {
    x: usize,
    y: usize,
}

struct FovStateT {
    source: OffsetT,
    mask: PermissiveMaskT,
    is_blocked: IsBlockedFunction,
    visit: VisitFunction,
    context: (),

    quadrant: OffsetT,
    extent: OffsetT,
}

struct LineT {}
