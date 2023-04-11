if not defined in_subprocess (cmd /k set in_subprocess=y ^& %0 %*) & exit )
battle_gui.exe map1 --server-rep-address tcp://0.0.0.0:4255 --server-bind-address tcp://0.0.0.0:4256 --side b --side-a-control W --side-a-control NW --side-a-control SW --side-b-control ALL
