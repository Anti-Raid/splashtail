package ext_types

type Permissions string

// Discordgo types are sometimes not of high quality, so we need to extend them
// with our own types [taken from serenity etc]. This is the place to do that.

/*
	A role tags object from serenity because discordgo doesnt actually support this

/// The Id of the bot the [`Role`] belongs to.

	pub bot_id: Option<UserId>,
	/// The Id of the integration the [`Role`] belongs to.
	pub integration_id: Option<IntegrationId>,
	/// Whether this is the guild's premium subscriber role.
	#[serde(default, skip_serializing_if = "is_false", with = "bool_as_option_unit")]
	pub premium_subscriber: bool,
	/// The id of this role's subscription sku and listing.
	pub subscription_listing_id: Option<SkuId>,
	/// Whether this role is available for purchase.
	#[serde(default, skip_serializing_if = "is_false", with = "bool_as_option_unit")]
	pub available_for_purchase: bool,
	/// Whether this role is a guild's linked role.
	#[serde(default, skip_serializing_if = "is_false", with = "bool_as_option_unit")]
	pub guild_connections: bool,
*/
type SerenityRoleTags struct {
	BotID                 *string `json:"bot_id" description:"The ID of the bot the role belongs to"`
	IntegrationID         *string `json:"integration_id" description:"The ID of the integration the role belongs to"`
	PremiumSubscriber     bool    `json:"premium_subscriber" description:"Whether this is the guild's premium subscriber role"`
	SubscriptionListingID *string `json:"subscription_listing_id" description:"The id of this role's subscription sku and listing"`
	AvailableForPurchase  bool    `json:"available_for_purchase" description:"Whether this role is available for purchase"`
	GuildConnections      bool    `json:"guild_connections" description:"Whether this role is a guild's linked role"`
}

/*
		A role object from serenity because discordgo's Role object is garbage

	    pub id: RoleId,
	    pub guild_id: GuildId,
	    pub colour: Colour,
	    pub name: FixedString<u32>,
	    pub permissions: Permissions,
	    pub position: i16,
	    pub tags: RoleTags,
	    pub icon: Option<ImageHash>,
	    pub unicode_emoji: Option<FixedString<u32>>,
*/
type SerenityRole struct {
	ID           string            `json:"id" description:"The ID of the role"`
	GuildID      string            `json:"guild_id" description:"The ID of the guild"`
	Color        int               `json:"color" description:"The color of the role"`
	Name         string            `json:"name" description:"The name of the role"`
	Permissions  *Permissions      `json:"permissions" description:"The permissions of the role"`
	Position     int16             `json:"position" description:"The position of the role"`
	Tags         *SerenityRoleTags `json:"tags" description:"The tags of the role"`
	Icon         *string           `json:"icon" description:"The icon of the role"`
	UnicodeEmoji string            `json:"unicode_emoji" description:"The unicode emoji of the role"`
}
