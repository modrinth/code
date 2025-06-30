---
title: Beginner's Guide to Licensing your Mods
summary: Software licenses; the nitty-gritty legal aspect of software development. They're more important than you think.
date: 2021-05-16
---

Why do you need to license your software? What are those licenses for anyway? These questions are more important than you think

## What is a software license?

To summarise the [Wikipedia article](https://en.wikipedia.org/wiki/Software_license) on the matter, it's essentially a legal contract between you (the mod developer) and anyone who uses, copies, modifies, etc the mod or any code having to do with it. License has the power to allow people to do whatever they want, or only permit the usage of the mod in-game. However, the majority of cases lie in-between these opposites.

## So which software license should I choose?

First and foremost, the choice of the software license is not entirely up to you, because you have to have the legal ability to do so. For instance, not all licenses are compatible with Minecraft's EULA (End-User License Agreement). Besides, if you are not the only one working on the project, you must get permission from all other contributors to your code before changing or adding a license. Please, ensure you have done so before implementing a license.

Before we can decide which one to use, however, we must establish some additional definitions. Open software licenses can be split into three main categories: **public domain**, **permissive**, and **copyleft**.

### Permissive license

A permissive license is a type of license that usually gives the abilities to use, copy, modify, distribute, sell, and relicense a piece of software.

The most popular license on Modrinth, the [MIT License](https://cdn.modrinth.com/licenses/mit.txt), is a permissive license. It is an easy-to-read license designed to be used for developers, which is why it is used extensively in the Minecraft open source community.

The [Apache License 2.0](https://cdn.modrinth.com/licenses/apache.txt) is also a very good permissive license to use. The main difference between it and the MIT License is that the Apache License gives an explicit patent grant, whereas patents must be registered manually with the MIT. There is also an additional clause with the Apache License, stating that any modified files must "carry prominent notices" of it being modified.

### Copyleft license

A copyleft license gives to the other party specific rights usually only given to the copyright owner, under the condition that those same rights are applied to all variations of that software. These are also sometimes called "viral" or "infectious" licenses, because of the requirement to pass those rights on to derivatives.

The second most common license on Modrinth is a copyleft license: the [GNU Lesser General Public License Version 3](https://cdn.modrinth.com/licenses/lgpl-3.txt) (usually [shortened to](https://spdx.org/licenses/LGPL-3.0-only.html) LGPL-3.0).

Typically, when a copyleft license is wanted, the [GPL-3.0](https://spdx.org/licenses/GPL-3.0-only.html) or [AGPL-3.0](https://spdx.org/licenses/AGPL-3.0-only.html) would be used. However, these licenses are **incompatible** if linking into Minecraft, due to an issue with the difference between proprietary and free software outlined by these licenses (more information [here](https://www.gnu.org/licenses/gpl-faq.html#GPLPlugins)). An exception can be added to allow linking, such as that found [here](https://gist.github.com/wafflecoffee/588f353802a3b0ea649e4fc85f75e583), but it is recommended to just use the LGPL-3.0 instead if possible.

### Public domain dedication

A public domain dedication gives all rights to everyone who gets a copy of the software. This includes but is not limited to the ability to use, copy, modify, distribute, sell, or relicense that software. Software with a public domain dedication has no copyright holder.

The third most common license used on Modrinth is the [Creative Commons Zero 1.0 Universal](https://cdn.modrinth.com/licenses/cc0.txt), which is a public domain dedication with a strong international legal basis, while still retaining trademark and patent rights.

Creative Commons licenses as a whole are not recommended for software, but rather for other creative works: use this license with caution. If you wish to have the simplest public domain dedication possible, the [Unlicense](https://cdn.modrinth.com/licenses/unlicense.txt) is also an option.

### What if I don't want to choose a license?

Without a license software is considered proprietary and all rights reserved. This means that people may only use it in the ways the copyright owner specifies, which, in the Minecraft world (no pun intended), typically just means downloading and using it; no modifications, unauthorized distributions: basically nothing.

This is why picking a proper software license is so important. It tells everyone what they can and cannot do with your software, making the difference between software anyone can contribute to and change however they want, and software that only you have the code behind.

That being said, All Rights Reserved and not using a license are options, if you don't want to choose a public domain, permissive, _or_ copyleft license. This can be useful in some cases, but as with any license, be aware of the effects: contributions will be difficult or impossible, and users may be inclined not to use your software. Also, in case of Minecraft, all mods, including the All Rights Reserved mods, are affected by Minecraft's EULA, which states:

> Any Mods you create for the Game from scratch belong to you (including pre-run Mods and in-memory Mods) and you can do whatever you want with them, as long as you don't sell them for money / try to make money from them and so long as you don't distribute Modded Versions of the Game.

What this means is you are not allowed to sell your mods even if you reserve all rights to them. There are plenty more examples of such details in licenses and other legal agreements in the modding world. All in all, be aware that you cannot decide all of your and other's rights with your license.

## Conclusion

To conclude, the importance of a software license cannot be overstated. You can choose whatever license you want (assuming you have the legal ability, of course), but be aware of the differences and consequences of choosing one over another. The licenses we've specified are what we recommend, as they are common and easy to understand. Hopefully, you will make your decision based on what you want to use and what your goals and purposes are.

A massive thank you goes to Alexander Ryckeboer (Progryck) for the cover image!

## Disclaimers

We are not lawyers, and thus, **this is not legal advice.** No warranty is given regarding this information, and we (Modrinth) disclaim liability for damages resulting in using this information given on an "as-is" basis. For more information on the legal aspect to software licensing, please refer to "[The Legal Side of Open Source](https://opensource.guide/legal/)".

No matter your choice of license, by uploading any Content (including but not limited to text, software, and graphics) to Modrinth, you give us certain rights to your Content, including but not limited to the ability to use, reproduce, or distribute. For more information, please see the [Modrinth Terms of Use](https://modrinth.com/legal/terms).

Measurements for "most popular license", "second most common license", and "third most common license", were taken 2021-04-30. Custom licenses were not taken into account.
