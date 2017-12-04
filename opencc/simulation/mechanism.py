# coding: utf-8
import typing

from opencc.const import SIDE, COLLECTION_ALIVE
from synergine2_xyz.subjects import XYZSubject
from synergine2_xyz.visible.simulation import VisibleMechanism


class OpponentVisibleMechanism(VisibleMechanism):
    from_collection = COLLECTION_ALIVE

    def reduce_subjects(self, subjects: typing.List[XYZSubject]) -> typing.Iterator[XYZSubject]:
        def filter_subject(subject: XYZSubject) -> bool:
            return self.subject.properties[SIDE] != subject.properties[SIDE]

        return filter(filter_subject, subjects)
