from cocos.director import director
from cocos.tiles import load
from cocos.layer import ScrollingManager
from cocos.scene import Scene

director.init()
a = load("/home/bux/Projets/OpenCC/opencc/maps/003/003.tmx")["top"]
scroller = ScrollingManager()
scroller.add(a)
director.run(Scene(scroller))
