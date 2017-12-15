# coding: utf-8
from synergine2_cocos2d.user_action import UserAction as BaseUserAction


class UserAction(BaseUserAction):
    ORDER_MOVE = 'ORDER_MOVE'
    ORDER_MOVE_FAST = 'ORDER_MOVE_FAST'
    ORDER_MOVE_CRAWL = 'ORDER_MOVE_CRAWL'
    ORDER_FIRE = 'ORDER_FIRE'
