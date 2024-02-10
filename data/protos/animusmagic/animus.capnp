# Note that this is still an experiment and will likely not come to fruition

@0xade7712499b71566; # Unique id created using `capnp id`

struct Message {
    data :union {
        probe @0 :Void;
        modules @1 :Void;

        guildsExist :group {
            guilds @2 :List(UInt64);
        }

        getBaseGuildAndUserInfo :group {
            guildId @3 :UInt64;
            userId @4 :UInt64;
        }
    }
}

struct Response {
    data :union {
        message :group {
            message @0 :Text;
            context @1 :Text;
        }

        test @2 :Void;
    }
}