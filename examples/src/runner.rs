use battle_core::map::Map;
use battle_gui::run::run;

pub struct Runner {
    map: Map,
    expire: Option<u64>,
}

impl Runner {
    pub fn run(mut self) {
        todo!()
        // run(
        //     opt,
        //     config,
        //     server_config,
        //     a_control,
        //     b_control,
        //     map,
        //     resources,
        //     deployment,
        //     battle_state,
        // )
    }
}

pub struct RunnerBuilder {
    map: Map,
    expire: Option<u64>,
}

impl RunnerBuilder {
    pub fn new(map: Map) -> Self {
        Self { map, expire: None }
    }
    pub fn expire(mut self, value: Option<u64>) -> Self {
        self.expire = value;
        self
    }

    pub fn build(self) -> Runner {
        Runner {
            map: self.map,
            expire: self.expire,
        }
    }
}
