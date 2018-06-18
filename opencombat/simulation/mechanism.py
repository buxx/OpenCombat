# coding: utf-8
import typing

from synergine2_xyz.subjects import XYZSubject
from synergine2_xyz.visible.simulation import VisibleMechanism
from synergine2.simulation import disable_when
from synergine2.simulation import config_value

from opencombat.const import SIDE
from opencombat.const import COLLECTION_ALIVE


class OpponentVisibleMechanism(VisibleMechanism):
    from_collection = COLLECTION_ALIVE

    @disable_when(config_value('_runtime.placement_mode'))
    def run(self) -> dict:
        return super().run()

    def reduce_subjects(self, subjects: typing.List[XYZSubject]) -> typing.Iterator[XYZSubject]:
        def filter_subject(subject: XYZSubject) -> bool:
            return self.subject.properties[SIDE] != subject.properties[SIDE]

        return filter(filter_subject, subjects)
