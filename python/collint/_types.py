from dataclasses import dataclass


@dataclass(frozen=True)
class TaggedBox:
    """A bounding box tagged with its clip box, semantic role, and id.

    Boxes are (x0, y0, x1, y1) in display pixels (origin bottom-left).
    """

    bx: tuple
    clip: tuple
    role: str
    id: str

    def as_tuple(self):
        return (self.bx, self.clip, self.role, self.id)
