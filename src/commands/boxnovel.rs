pub mod boxnovel {
    use serenity::framework::standard::{
        CommandResult,
        macros::{
            command,
            group,
        },
        StandardFramework,
    };
    use serenity::prelude::{Context, EventHandler};
    use serenity::model::prelude::Message;
    use serenity::framework::standard::{
        Args,
    };
    use crate::Db;


    #[group]
    #[prefixes(bn, boxnovel)]
    #[commands(add)]
    struct Boxnovel;

    #[command]
    async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
        let mut data = ctx.data.read().await;
        let db = data.get::<Db>().unwrap();

        let guild_id = msg.guild_id.unwrap().to_string();
        let channel_id = msg.channel_id.to_string();

        let novel: String = args.single::<String>()?;

        Ok(())
    }
}
