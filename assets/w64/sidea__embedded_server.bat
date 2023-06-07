if not defined in_subprocess (cmd /k set in_subprocess=y ^& %0 %*) & exit )
battle_gui.exe Demo1 assets/demo1_deployment.json --embedded-server --server-rep-address tcp://0.0.0.0:4255 --server-bind-address tcp://0.0.0.0:4256 --side a --side-a-control N --side-a-control NW --side-a-control W --side-b-control ALL
