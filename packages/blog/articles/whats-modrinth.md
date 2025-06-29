---
title: What is Modrinth?
summary: "Hello, we are Modrinth – an open source mods hosting platform. Sounds dry, doesn't it? So let me tell you our story – and I promise, it won't be boring!"
date: 2020-11-27
---

Hello, we are Modrinth – an open source mods hosting platform. Sounds dry, doesn't it? So let me tell you our story – and I promise, it won't be boring!

## Prelude and conception

Before Modrinth was even thought of, there already were several giant platforms for mod hosting. However, most of them were too specialized, outdated, or transitively dependent on an uncaring hegemonic 3rd party. Authors and players were always in struggle. The community had to choose 2 out of 3: inconvenience, indifference, obsolescence. Urge for better service, either new or renewed, just founded or acquired arose.

Although demand for proper competition is the seed, the germ of Modrinth, the biggest role was played by the Fabric project. It set an example of a community-powered alternative. It was democratic, FOSS, listening to the community, and welcoming contribution and 3rd party initiatives. They have shown the modding community that they can evolve and adapt, be accessible and welcoming, cooperative, and caring.

## Fabricate and HexFabric

And, oh boy, did they connect – the demand for competition grew so high, that at some point the community just exploded with novelty. During several months, almost a dozen projects were aiming to be the second Walmart, the third IKEA, the fourth Amazon for your mods. Here beings the story of HexFabrics... – wait, what? What's that?

> HexFabric is an umbrella term for modern mod hosting technology. It got its name from Fabric, which at the point was poorly supported (if at all) by the major players on the stage. In practice, HexFabric is just a cozy Discord server, on which several projects have their deputizing channels.

Back on track – Lots of HexFabrics were founded almost simultaneously. Altar.gg, Astronave, Diluv, ModForest, Minerepo... and, most importantly, Fabricate.

Fabricate began its journey as a proprietary project indexing website by a single developer – Geometrically. It remained relatively unnoticed for a couple of weeks, and then it started gaining attention. This new website has amazing search! Yup, the whole thing was primarily about making seamless, gracious, appeasing smart real-time search. The community is now intrigued.

## Becoming a team

"But this looks awful! And it's proprietary!" – a few voices said. Among those voices were falseresync and MulverineX. They both had several objections to that and were pestering the original author. "FOSS is the true way for community project" and "Just use a license to prevent others from creating instances of your work," they told.

Yet Fabricate remained proprietary for a while. However, once the pressure on the author became high, they gave up and open-sourced their work. This was the birth of Modrinth. It did not get its name for a little while longer though.

Now that Modrinth was open source, it started gaining traction. Remember falseresync and MulverineX? They joined Geometrically on the branding site, and somewhere in the middle of the brainstorming process the logo and the name were born. At the same time AppleTheGolden, Aeledfyr, and Redblueflame began contributing to the actual code of the project, which is – nowadays known to everyone – in Rust. A solo suddenly became a team, ready for whatever future holds.

## Development non-stop

The newly born FOSS project is now evolving swiftly. Before our team arose the question: monolithic vs split app architecture. Monolithic would be easier to deploy and can serve pages quicker. The split architecture will simplify the development and allow for a feature-full user experience. The discussion was hot, and the sides were fierce. Nevertheless, the split pattern won. Now it was time to make proper backend and frontend apps.

The work first began with the backend. Aeled, Red, and Geo started detaching API methods from visuals. The team worked hard. Consequent to the API splitting from the GUI, it became getting new and exciting features. The first feature to be added was custom Modrinth mods – before that, the website only indexed the competitor's service.

However, for that to happen there had to be another step taken – migration from MongoDB to PostgreSQL. It was crucial for efficient data storage and complex relationships between projects. And the biggest propagator of that change was Apple, who introduced and successfully defended their case.

Thus, with custom mods, better yet search has been implemented. After search, user accounts with external log-in made their way into the project. Now it first creators started uploading their mods – a monumental achievement.

After the first creators came more – the community began taking Modrinth as a serious alternative hosting. At some point, uploads accelerated to the point that our team was forced to redo their plans and establish project editing and moderation considerably earlier than it could have been. Besides, creators need analytics, they need teams, they needed support system. So the backend developers tried their best to keep up and achieved their goals through enthusiastic labor and dedication.

## Refreshed look

Although, on the frontend side things weren't as bright, unfortunately. Once falseresync presented the new look and feel Modrinth should aim for, he was forced to dedicate less time to the project. As a consequence, the frontend was implemented rather haphazardly and was lacking in features compared to the backend.

However, this did not stop the project from evolving. The backend team has continued to expand on existing features, and after a long period of time, the savior descended on the frontend – Prospector, who rapidly became a crucial contributor and a part of the team. With new and comprehensive design guidance from falseresync and critique from MulverineX and the community, Prospector achieved feature parity with backend and greatly improved the website look and feel.

Improving the frontend wasn't an easy job: naughty CSS, runtime errors, the abundance of framework-related nuances – all were obstacles, and all were defeated. Through battles with web technologies, jokes about quirky styles, and hard work our team created the UI you see today.

## Going beta

> You have to believe that the dots will somehow connect in your future. _– Steve Jobs_

With the story complete, we are proud to announce that the Modrinth beta will be coming out on November 30th, with a refreshed look and a feature-complete modding website! It is a tremendous achievement for us and the community, which we are very proud of.

It is heart-warming to admit that we're finally going officially online. We know it's not perfect yet. But regardless, we will continue our passion project as a team, and we will expand on it and make it only better!

Stay tuned!
