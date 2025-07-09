---
title: Configuring Badgerfang
description: How to guide on configuring Badgerfang
---

# Basics

While most of the Image-Based assets are located at `/public` folder, those aren't the only things you'll need to edit if you want to completely self host this website.

For Instance, the [commons](https://github.com/Anti-Raid/badgerfang-next/blob/production/src/components/common.ts) located in the components provide's all the meta-data information. This can be edited so lets say if you want to add more keywords then this is your file!

For API related config change you can make your way to the [API]() library. This provides the Endpoint URLS for all the fetches etc thats used within the website.

```ts lineNumbers=14
const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL || 'https://splashtail-staging.antiraid.xyz';
const FORUM_API_URL = 'https://potsypaw.purrquinox.com';
const STRAPI_API_URL = 'https://strapi.purrquinox.com';
```