<div id="@antiraid/datetime"></div>

# @antiraid/datetime

<div id="Types"></div>

## Types

<div id="TimeDelta"></div>

## TimeDelta

<details>
<summary>Raw Type</summary>

```luau
type TimeDelta = {
	--- @field nanos: The number of nanoseconds in the time delta.
	nanos: number,

	--- @field micros: The number of microseconds in the time delta.
	micros: number,

	--- @field millis: The number of milliseconds in the time delta.
	millis: number,

	--- @field seconds: The number of seconds in the time delta.
	seconds: number,

	--- @field minutes: The number of minutes in the time delta.
	minutes: number,

	--- @field hours: The number of hours in the time delta.
	hours: number,

	--- @field days: The number of days in the time delta.
	days: number,

	--- @field weeks: The number of weeks in the time delta.
	weeks: number,

	--- @function () -> string
	--- Returns an 'offset' string representation of the time delta.
	--- E.g. "+05:30" for 5 hours and 30 minutes.
	offset_string: (self: TimeDelta) -> string,

	-- Metatable
	__add: (TimeDelta, TimeDelta) -> TimeDelta,
	__sub: (TimeDelta, TimeDelta) -> TimeDelta,
	__le: (TimeDelta, TimeDelta) -> boolean,
	__lt: (TimeDelta, TimeDelta) -> boolean,
	__tostring: (TimeDelta) -> string
}
```

</details>

<div id="nanos"></div>

### nanos

[number](#number)

<div id="micros"></div>

### micros

[number](#number)

<div id="millis"></div>

### millis

[number](#number)

<div id="seconds"></div>

### seconds

[number](#number)

<div id="minutes"></div>

### minutes

[number](#number)

<div id="hours"></div>

### hours

[number](#number)

<div id="days"></div>

### days

[number](#number)

<div id="weeks"></div>

### weeks

[number](#number)

<div id="offset_string"></div>

### offset_string

@function () -> string

Returns an 'offset' string representation of the time delta.

E.g. "+05:30" for 5 hours and 30 minutes.

<details>
<summary>Function Signature</summary>

```luau
--- @function () -> string
--- Returns an 'offset' string representation of the time delta.
--- E.g. "+05:30" for 5 hours and 30 minutes.
offset_string: (self: TimeDelta) -> string
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[string](#string)<div id="MetatableFields"></div>

### Metatable Fields

<div id="__add"></div>

#### __add

<details>
<summary>Function Signature</summary>

```luau
__add: (TimeDelta, TimeDelta) -> TimeDelta
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="arg1"></div>

##### arg1

[TimeDelta](#TimeDelta)

<div id="arg2"></div>

##### arg2

[TimeDelta](#TimeDelta)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[TimeDelta](#TimeDelta)<div id="__sub"></div>

#### __sub

<details>
<summary>Function Signature</summary>

```luau
__sub: (TimeDelta, TimeDelta) -> TimeDelta
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="arg1"></div>

##### arg1

[TimeDelta](#TimeDelta)

<div id="arg2"></div>

##### arg2

[TimeDelta](#TimeDelta)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[TimeDelta](#TimeDelta)<div id="__le"></div>

#### __le

<details>
<summary>Function Signature</summary>

```luau
__le: (TimeDelta, TimeDelta) -> boolean
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="arg1"></div>

##### arg1

[TimeDelta](#TimeDelta)

<div id="arg2"></div>

##### arg2

[TimeDelta](#TimeDelta)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[boolean](#boolean)<div id="__lt"></div>

#### __lt

<details>
<summary>Function Signature</summary>

```luau
__lt: (TimeDelta, TimeDelta) -> boolean
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="arg1"></div>

##### arg1

[TimeDelta](#TimeDelta)

<div id="arg2"></div>

##### arg2

[TimeDelta](#TimeDelta)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[boolean](#boolean)<div id="__tostring"></div>

#### __tostring

<details>
<summary>Function Signature</summary>

```luau
__tostring: (TimeDelta) -> string
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="arg1"></div>

##### arg1

[TimeDelta](#TimeDelta)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[string](#string)<div id="TimeZone"></div>

## TimeZone

@class TimeZone

@within TimeZone

A timezone object.

<details>
<summary>Raw Type</summary>

```luau
--- @class TimeZone
--- @within TimeZone
--- A timezone object.
type TimeZone = {
	--- Parses a datetime string and returns a DateTime object.
	fromString: (self: TimeZone, datetime: string) -> DateTime,

	--- @function (year: number, month: number, day: number, hour: number, minute: number, second: number, all: boolean?) -> DateTime
	--- Translates a timestamp in UTC time to a datetime in the said specific timezone. If `all` is set to true, then multiple times
	--- may be returned in the case of ambiguous times, otherwise the first time is returned.
	utcToTz: (self: TimeZone, year: number, month: number, day: number, hours: number, minutes: number, secs: number) -> DateTime,

	--- @function (year: number, month: number, day: number, hour: number, minute: number, second: number, all: boolean?) -> DateTime
	--- Translates a timestamp in the specified timezone to a datetime in UTC. If `all` is set to true, then multiple times
	--- may be returned in the case of ambiguous times, otherwise the first time is returned.
	tzToUtc: (self: TimeZone, year: number, month: number, day: number, hours: number, minutes: number, secs: number) -> DateTime,

	--- @function (hour: number, minute: number, second: number) -> DateTime
	--- Translates a time of the current day in UTC time to a datetime in the said specific timezone
	timeUtcToTz: (self: TimeZone, hours: number, minutes: number, secs: number) -> DateTime,

	--- @function (hour: number, minute: number, second: number) -> DateTime
	--- Translates a time of the current day in the said specific timezone to a datetime in UTC
	timeTzToUtc: (self: TimeZone, hours: number, minutes: number, secs: number) -> DateTime,

	--- @function () -> DateTime
	--- Translates the current timestamp to a datetime in the said specific timezone
	now: (self: TimeZone) -> DateTime,

	--- Converts a unix timestamp to a datetime in the said specific timezone
	fromTime: (self: TimeZone, timestamp: typesext.I64Convertibles) -> DateTime,

	--- Converts a unix timestamp in milliseconds to a datetime in the said specific timezone
	fromTimeMillis: (self: TimeZone, timestamp: typesext.I64Convertibles) -> DateTime,

	--- Converts a unix timestamp in microseconds to a datetime in the said specific timezone
	fromTimeMicros: (self: TimeZone, timestamp: typesext.I64Convertibles) -> DateTime,

	-- Metatable
	__tostring: (TimeZone) -> string,
	__eq: (TimeZone, TimeZone) -> boolean
}
```

</details>

<div id="fromString"></div>

### fromString

Parses a datetime string and returns a DateTime object.

<details>
<summary>Function Signature</summary>

```luau
--- Parses a datetime string and returns a DateTime object.
fromString: (self: TimeZone, datetime: string) -> DateTime
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="datetime"></div>

##### datetime

[string](#string)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[DateTime](#DateTime)<div id="utcToTz"></div>

### utcToTz

@function (year: number, month: number, day: number, hour: number, minute: number, second: number, all: boolean?) -> DateTime

Translates a timestamp in UTC time to a datetime in the said specific timezone. If `all` is set to true, then multiple times

may be returned in the case of ambiguous times, otherwise the first time is returned.

<details>
<summary>Function Signature</summary>

```luau
--- @function (year: number, month: number, day: number, hour: number, minute: number, second: number, all: boolean?) -> DateTime
--- Translates a timestamp in UTC time to a datetime in the said specific timezone. If `all` is set to true, then multiple times
--- may be returned in the case of ambiguous times, otherwise the first time is returned.
utcToTz: (self: TimeZone, year: number, month: number, day: number, hours: number, minutes: number, secs: number) -> DateTime
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="year"></div>

##### year

[number](#number)

<div id="month"></div>

##### month

[number](#number)

<div id="day"></div>

##### day

[number](#number)

<div id="hours"></div>

##### hours

[number](#number)

<div id="minutes"></div>

##### minutes

[number](#number)

<div id="secs"></div>

##### secs

[number](#number)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[DateTime](#DateTime)<div id="tzToUtc"></div>

### tzToUtc

@function (year: number, month: number, day: number, hour: number, minute: number, second: number, all: boolean?) -> DateTime

Translates a timestamp in the specified timezone to a datetime in UTC. If `all` is set to true, then multiple times

may be returned in the case of ambiguous times, otherwise the first time is returned.

<details>
<summary>Function Signature</summary>

```luau
--- @function (year: number, month: number, day: number, hour: number, minute: number, second: number, all: boolean?) -> DateTime
--- Translates a timestamp in the specified timezone to a datetime in UTC. If `all` is set to true, then multiple times
--- may be returned in the case of ambiguous times, otherwise the first time is returned.
tzToUtc: (self: TimeZone, year: number, month: number, day: number, hours: number, minutes: number, secs: number) -> DateTime
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="year"></div>

##### year

[number](#number)

<div id="month"></div>

##### month

[number](#number)

<div id="day"></div>

##### day

[number](#number)

<div id="hours"></div>

##### hours

[number](#number)

<div id="minutes"></div>

##### minutes

[number](#number)

<div id="secs"></div>

##### secs

[number](#number)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[DateTime](#DateTime)<div id="timeUtcToTz"></div>

### timeUtcToTz

@function (hour: number, minute: number, second: number) -> DateTime

Translates a time of the current day in UTC time to a datetime in the said specific timezone

<details>
<summary>Function Signature</summary>

```luau
--- @function (hour: number, minute: number, second: number) -> DateTime
--- Translates a time of the current day in UTC time to a datetime in the said specific timezone
timeUtcToTz: (self: TimeZone, hours: number, minutes: number, secs: number) -> DateTime
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="hours"></div>

##### hours

[number](#number)

<div id="minutes"></div>

##### minutes

[number](#number)

<div id="secs"></div>

##### secs

[number](#number)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[DateTime](#DateTime)<div id="timeTzToUtc"></div>

### timeTzToUtc

@function (hour: number, minute: number, second: number) -> DateTime

Translates a time of the current day in the said specific timezone to a datetime in UTC

<details>
<summary>Function Signature</summary>

```luau
--- @function (hour: number, minute: number, second: number) -> DateTime
--- Translates a time of the current day in the said specific timezone to a datetime in UTC
timeTzToUtc: (self: TimeZone, hours: number, minutes: number, secs: number) -> DateTime
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="hours"></div>

##### hours

[number](#number)

<div id="minutes"></div>

##### minutes

[number](#number)

<div id="secs"></div>

##### secs

[number](#number)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[DateTime](#DateTime)<div id="now"></div>

### now

@function () -> DateTime

Translates the current timestamp to a datetime in the said specific timezone

<details>
<summary>Function Signature</summary>

```luau
--- @function () -> DateTime
--- Translates the current timestamp to a datetime in the said specific timezone
now: (self: TimeZone) -> DateTime
```

</details>

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[DateTime](#DateTime)<div id="fromTime"></div>

### fromTime

Converts a unix timestamp to a datetime in the said specific timezone

<details>
<summary>Function Signature</summary>

```luau
--- Converts a unix timestamp to a datetime in the said specific timezone
fromTime: (self: TimeZone, timestamp: typesext.I64Convertibles) -> DateTime
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="timestamp"></div>

##### timestamp

[typesext](./typesext.md).[I64Convertibles](./typesext.md#I64Convertibles)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[DateTime](#DateTime)<div id="fromTimeMillis"></div>

### fromTimeMillis

Converts a unix timestamp in milliseconds to a datetime in the said specific timezone

<details>
<summary>Function Signature</summary>

```luau
--- Converts a unix timestamp in milliseconds to a datetime in the said specific timezone
fromTimeMillis: (self: TimeZone, timestamp: typesext.I64Convertibles) -> DateTime
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="timestamp"></div>

##### timestamp

[typesext](./typesext.md).[I64Convertibles](./typesext.md#I64Convertibles)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[DateTime](#DateTime)<div id="fromTimeMicros"></div>

### fromTimeMicros

Converts a unix timestamp in microseconds to a datetime in the said specific timezone

<details>
<summary>Function Signature</summary>

```luau
--- Converts a unix timestamp in microseconds to a datetime in the said specific timezone
fromTimeMicros: (self: TimeZone, timestamp: typesext.I64Convertibles) -> DateTime
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="timestamp"></div>

##### timestamp

[typesext](./typesext.md).[I64Convertibles](./typesext.md#I64Convertibles)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[DateTime](#DateTime)<div id="MetatableFields"></div>

### Metatable Fields

<div id="__tostring"></div>

#### __tostring

<details>
<summary>Function Signature</summary>

```luau
__tostring: (TimeZone) -> string
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="arg1"></div>

##### arg1

[TimeZone](#TimeZone)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[string](#string)<div id="__eq"></div>

#### __eq

<details>
<summary>Function Signature</summary>

```luau
__eq: (TimeZone, TimeZone) -> boolean
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="arg1"></div>

##### arg1

[TimeZone](#TimeZone)

<div id="arg2"></div>

##### arg2

[TimeZone](#TimeZone)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[boolean](#boolean)<div id="DateTime"></div>

## DateTime

@class DateTime

@within DateTime

A datetime object.



Supports addition/subtraction/equality with TimeDelta objects as well as comparisons with other DateTime objects.

<details>
<summary>Raw Type</summary>

```luau
--- @class DateTime
--- @within DateTime
--- A datetime object. 
---
--- Supports addition/subtraction/equality with TimeDelta objects as well as comparisons with other DateTime objects.
type DateTime = {
	--- @field year
	--- The year of the datetime.
	year: number,

	--- @field month
	--- The month of the datetime.
	month: number,

	--- @field day
	--- The day of the datetime.
	day: number,

	--- @field hour
	--- The hour of the datetime.
	hour: number,

	--- @field minute
	--- The minute of the datetime.
	minute: number,

	--- @field second
	--- The second of the datetime.
	second: number,

	--- @field timestamp_seconds
	--- The timestamp in seconds of the datetime from the Unix epoch.
	timestamp_seconds: number,

	--- @field timestamp_millis
	--- The timestamp in milliseconds of the datetime from the Unix epoch.
	timestamp_millis: number,

	--- @field timestamp_micros
	--- The timestamp in microseconds of the datetime from the Unix epoch.
	timestamp_micros: number,

	--- @field timestamp_nanos
	--- The timestamp in nanoseconds of the datetime from the Unix epoch.
	timestamp_nanos: number,

	--- @field timezone: TimeZone
	--- The timezone of the datetime.
	timezone: TimeZone,

	--- @field base_offset: TimeDelta
	--- The base (non-DST) offset of the datetime.
	base_offset: TimeDelta,

	--- @field dst_offset: TimeDelta
	--- The additional DST offset of the datetime.
	dst_offset: TimeDelta,

	--- @function (TimeZone) -> DateTime
	--- Converts the datetime to the specified timezone.
	with_timezone: (self: TimeZone, TimeZone) -> DateTime,

	--- @function (string) -> string
	--- Formats the datetime using the specified format string.
	format: (self: TimeZone, string) -> string,

	--- @function (DateTime) -> TimeDelta
	--- Calculates the duration between the current datetime and another datetime.
	duration_since: (self: TimeZone, DateTime) -> TimeDelta,

	-- Metatable
	__add: (DateTime, TimeDelta) -> DateTime,
	__sub: (DateTime, TimeDelta) -> DateTime,
	__le: (DateTime, DateTime) -> boolean,
	__lt: (DateTime, DateTime) -> boolean,
	__eq: (DateTime, DateTime) -> boolean,
	__tostring: (DateTime) -> string
}
```

</details>

<div id="year"></div>

### year

The year of the datetime.

[number](#number)

<div id="month"></div>

### month

The month of the datetime.

[number](#number)

<div id="day"></div>

### day

The day of the datetime.

[number](#number)

<div id="hour"></div>

### hour

The hour of the datetime.

[number](#number)

<div id="minute"></div>

### minute

The minute of the datetime.

[number](#number)

<div id="second"></div>

### second

The second of the datetime.

[number](#number)

<div id="timestamp_seconds"></div>

### timestamp_seconds

The timestamp in seconds of the datetime from the Unix epoch.

[number](#number)

<div id="timestamp_millis"></div>

### timestamp_millis

The timestamp in milliseconds of the datetime from the Unix epoch.

[number](#number)

<div id="timestamp_micros"></div>

### timestamp_micros

The timestamp in microseconds of the datetime from the Unix epoch.

[number](#number)

<div id="timestamp_nanos"></div>

### timestamp_nanos

The timestamp in nanoseconds of the datetime from the Unix epoch.

[number](#number)

<div id="timezone"></div>

### timezone

The timezone of the datetime.

[TimeZone](#TimeZone)

<div id="base_offset"></div>

### base_offset

The base (non-DST) offset of the datetime.

[TimeDelta](#TimeDelta)

<div id="dst_offset"></div>

### dst_offset

The additional DST offset of the datetime.

[TimeDelta](#TimeDelta)

<div id="with_timezone"></div>

### with_timezone

@function (TimeZone) -> DateTime

Converts the datetime to the specified timezone.

<details>
<summary>Function Signature</summary>

```luau
--- @function (TimeZone) -> DateTime
--- Converts the datetime to the specified timezone.
with_timezone: (self: TimeZone, TimeZone) -> DateTime
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="arg2"></div>

##### arg2

[TimeZone](#TimeZone)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[DateTime](#DateTime)<div id="format"></div>

### format

@function (string) -> string

Formats the datetime using the specified format string.

<details>
<summary>Function Signature</summary>

```luau
--- @function (string) -> string
--- Formats the datetime using the specified format string.
format: (self: TimeZone, string) -> string
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="arg2"></div>

##### arg2

[string](#string)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[string](#string)<div id="duration_since"></div>

### duration_since

@function (DateTime) -> TimeDelta

Calculates the duration between the current datetime and another datetime.

<details>
<summary>Function Signature</summary>

```luau
--- @function (DateTime) -> TimeDelta
--- Calculates the duration between the current datetime and another datetime.
duration_since: (self: TimeZone, DateTime) -> TimeDelta
```

</details>

<div id="Arguments"></div>

#### Arguments

<div id="arg2"></div>

##### arg2

[DateTime](#DateTime)

<div id="Returns"></div>

#### Returns

<div id="ret1"></div>

##### ret1

[TimeDelta](#TimeDelta)<div id="MetatableFields"></div>

### Metatable Fields

<div id="__add"></div>

#### __add

<details>
<summary>Function Signature</summary>

```luau
__add: (DateTime, TimeDelta) -> DateTime
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="arg1"></div>

##### arg1

[DateTime](#DateTime)

<div id="arg2"></div>

##### arg2

[TimeDelta](#TimeDelta)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[DateTime](#DateTime)<div id="__sub"></div>

#### __sub

<details>
<summary>Function Signature</summary>

```luau
__sub: (DateTime, TimeDelta) -> DateTime
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="arg1"></div>

##### arg1

[DateTime](#DateTime)

<div id="arg2"></div>

##### arg2

[TimeDelta](#TimeDelta)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[DateTime](#DateTime)<div id="__le"></div>

#### __le

<details>
<summary>Function Signature</summary>

```luau
__le: (DateTime, DateTime) -> boolean
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="arg1"></div>

##### arg1

[DateTime](#DateTime)

<div id="arg2"></div>

##### arg2

[DateTime](#DateTime)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[boolean](#boolean)<div id="__lt"></div>

#### __lt

<details>
<summary>Function Signature</summary>

```luau
__lt: (DateTime, DateTime) -> boolean
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="arg1"></div>

##### arg1

[DateTime](#DateTime)

<div id="arg2"></div>

##### arg2

[DateTime](#DateTime)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[boolean](#boolean)<div id="__eq"></div>

#### __eq

<details>
<summary>Function Signature</summary>

```luau
__eq: (DateTime, DateTime) -> boolean
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="arg1"></div>

##### arg1

[DateTime](#DateTime)

<div id="arg2"></div>

##### arg2

[DateTime](#DateTime)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[boolean](#boolean)<div id="__tostring"></div>

#### __tostring

<details>
<summary>Function Signature</summary>

```luau
__tostring: (DateTime) -> string
```

</details>

<div id="Arguments"></div>

##### Arguments

<div id="arg1"></div>

##### arg1

[DateTime](#DateTime)

<div id="Returns"></div>

##### Returns

<div id="ret1"></div>

##### ret1

[string](#string)<div id="Functions"></div>

# Functions

<div id="new"></div>

## new

@function (timezone: string) -> TimeZone

Returns a new Timezone object if the timezone is recognized/supported.

<details>
<summary>Function Signature</summary>

```luau
--- @function (timezone: string) -> TimeZone
--- @param timezone: string (The timezone to get the offset for.)
--- @return TimeZone (The timezone object.)
--- Returns a new Timezone object if the timezone is recognized/supported.
function new(timezone: string) -> TimeZone end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="timezone"></div>

### timezone

[string](#string)

<div id="Returns"></div>

## Returns

<div id="TimeZone"></div>

### TimeZone

(The timezone object.)

[TimeZone](#TimeZone)<div id="timedelta_weeks"></div>

## timedelta_weeks

@function (weeks: number) -> TimeDelta

Creates a new TimeDelta object with the specified number of weeks.

<details>
<summary>Function Signature</summary>

```luau
--- @function (weeks: number) -> TimeDelta
--- @param weeks: number (The number of weeks to create the TimeDelta object with.)
--- @return TimeDelta
--- Creates a new TimeDelta object with the specified number of weeks.
function timedelta_weeks(weeks: typesext.I64Convertibles) -> TimeDelta end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="weeks"></div>

### weeks

[typesext](./typesext.md).[I64Convertibles](./typesext.md#I64Convertibles)

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[TimeDelta](#TimeDelta)<div id="timedelta_days"></div>

## timedelta_days

@function (days: number) -> TimeDelta

Creates a new TimeDelta object with the specified number of days.

<details>
<summary>Function Signature</summary>

```luau
--- @function (days: number) -> TimeDelta
--- @param days: number (The number of days to create the TimeDelta object with.)
--- @return TimeDelta
--- Creates a new TimeDelta object with the specified number of days.
function timedelta_days(days: typesext.I64Convertibles) -> TimeDelta end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="days"></div>

### days

[typesext](./typesext.md).[I64Convertibles](./typesext.md#I64Convertibles)

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[TimeDelta](#TimeDelta)<div id="timedelta_hours"></div>

## timedelta_hours

@function (hours: number) -> TimeDelta

Creates a new TimeDelta object with the specified number of hours.

<details>
<summary>Function Signature</summary>

```luau
--- @function (hours: number) -> TimeDelta
--- @param hours: number (The number of hours to create the TimeDelta object with.)
--- @return TimeDelta
--- Creates a new TimeDelta object with the specified number of hours.
function timedelta_hours(hours: typesext.I64Convertibles) -> TimeDelta end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="hours"></div>

### hours

[typesext](./typesext.md).[I64Convertibles](./typesext.md#I64Convertibles)

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[TimeDelta](#TimeDelta)<div id="timedelta_minutes"></div>

## timedelta_minutes

@function (minutes: number) -> TimeDelta

Creates a new TimeDelta object with the specified number of minutes.

<details>
<summary>Function Signature</summary>

```luau
--- @function (minutes: number) -> TimeDelta
--- @param minutes: number (The number of minutes to create the TimeDelta object with.)
--- @return TimeDelta
--- Creates a new TimeDelta object with the specified number of minutes.
function timedelta_minutes(minutes: typesext.I64Convertibles) -> TimeDelta end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="minutes"></div>

### minutes

[typesext](./typesext.md).[I64Convertibles](./typesext.md#I64Convertibles)

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[TimeDelta](#TimeDelta)<div id="timedelta_seconds"></div>

## timedelta_seconds

@function (seconds: number) -> TimeDelta

Creates a new TimeDelta object with the specified number of seconds.

<details>
<summary>Function Signature</summary>

```luau
--- @function (seconds: number) -> TimeDelta
--- @param seconds: number (The number of seconds to create the TimeDelta object with.)
--- @return TimeDelta
--- Creates a new TimeDelta object with the specified number of seconds.
function timedelta_seconds(seconds: typesext.I64Convertibles) -> TimeDelta end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="seconds"></div>

### seconds

[typesext](./typesext.md).[I64Convertibles](./typesext.md#I64Convertibles)

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[TimeDelta](#TimeDelta)<div id="timedelta_millis"></div>

## timedelta_millis

@function (millis: number) -> TimeDelta

Creates a new TimeDelta object with the specified number of milliseconds.

<details>
<summary>Function Signature</summary>

```luau
--- @function (millis: number) -> TimeDelta
--- @param millis: number (The number of milliseconds to create the TimeDelta object with.)
--- @return TimeDelta
--- Creates a new TimeDelta object with the specified number of milliseconds.
function timedelta_millis(millis: typesext.I64Convertibles) -> TimeDelta end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="millis"></div>

### millis

[typesext](./typesext.md).[I64Convertibles](./typesext.md#I64Convertibles)

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[TimeDelta](#TimeDelta)<div id="timedelta_micros"></div>

## timedelta_micros

@function (micros: number) -> TimeDelta

Creates a new TimeDelta object with the specified number of microseconds.

<details>
<summary>Function Signature</summary>

```luau
--- @function (micros: number) -> TimeDelta
--- @param micros: number (The number of microseconds to create the TimeDelta object with.)
--- @return TimeDelta
--- Creates a new TimeDelta object with the specified number of microseconds.
function timedelta_micros(micros: typesext.I64Convertibles) -> TimeDelta end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="micros"></div>

### micros

[typesext](./typesext.md).[I64Convertibles](./typesext.md#I64Convertibles)

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[TimeDelta](#TimeDelta)<div id="timedelta_nanos"></div>

## timedelta_nanos

@function (nanos: number) -> TimeDelta

Creates a new TimeDelta object with the specified number of nanoseconds.

<details>
<summary>Function Signature</summary>

```luau
--- @function (nanos: number) -> TimeDelta
--- @param nanos: number (The number of nanoseconds to create the TimeDelta object with.)
--- @return TimeDelta
--- Creates a new TimeDelta object with the specified number of nanoseconds.
function timedelta_nanos(nanos: typesext.I64Convertibles) -> TimeDelta end
```

</details>

<div id="Arguments"></div>

## Arguments

<div id="nanos"></div>

### nanos

[typesext](./typesext.md).[I64Convertibles](./typesext.md#I64Convertibles)

<div id="Returns"></div>

## Returns

<div id="ret1"></div>

### ret1

[TimeDelta](#TimeDelta)