# ice-repos

My goal here is to build a simple web-app allowing you to batch
archive groups of repositories from a given organization, using
[Rust](https://www.rust-lang.org)+[Yew](https://yew.rs).

As [a university faculty in computer science](https://academics.morris.umn.edu/nic-mcphee),
I end up being the
(co-)owner of dozens (often over 100) [student repositories](https://github.com/UMM-CSci-3601-S22)
a year that are created via GitHub Classroom. I would like to
archive these at the end of each semester or school year for
a variety of reasons:

- It would help make it clear to folks that stumble across them
  that they're not under active development.
- I wouldn't keep getting notifications (often _many_ notifications)
  from tools like [Dependabot](https://github.com/dependabot) because repositories from old
  courses have a dependency with a potential security concern.

[Archiving a single repository via the GitHub web interface](https://docs.github.com/en/repositories/archiving-a-github-repository/archiving-repositories)
is reasonable enough, but archiving dozens of repositories by hand
is _incredibly_ tedious and really not feasible. It's pretty clear
that [this is doable via the GitHub API](https://docs.github.com/en/rest/repos/repos),
though, so I'd like to
build a simple web app that allows me (and any other folks that
might find this helpful) to specify an organization (e.g., the
organization for a given semester of a course), and then get
a list of the (not yet archived) repos in that organization, with the
option of selecting which ones we'd like to actually archive.

My current plan is to do this as a Rust+Yew WASM app. There's
really no particular reason to use Rust for this, but gaining
a better understanding of Rust is one of my Year of Programming
goals, so why not use it here?

I'll start by just trying to get a basic system up and running,
but there are some things I might be interested in experimenting
with as we go along:

- Use [Cypress for E2E testing](https://www.cypress.io) on the
  app. Probably don't really
  need it here, because the interface and functionality will be
  pretty simple, but it might be interesting to see how that
  plays out here. A downside, though, is that will bring the
  whole JS/npm ecosystem into the project, which is a pain.
  I don't know anything comparable that doesn't, though. I'll
  start with just Rust unit testing, but if I feel a compelling
  urge to bring in E2E testing then we'll rent a big tractor and
  drag Cypress into the picture. [I did a little experiment](https://github.com/NicMcPhee/rust-yew-cypress)
  to convince myself that it's doable, and it totally is, so we'll
  see.
- See if I can get some kind of code coverage report, preferably
  tht can be included in the build checks via GitHub Actions.
- Deploy the app on [GitHub pages](https://pages.github.com).
  I don't think there's any
  reason that this can't work fine as a serverless app, and if
  that does work then I can make this available to other interested
  folks (e.g., other instructors, people retiring old organizations)
  without any deployment cost to me. There seems to be [a
  template that may help set up a lot of what we want](https://github.com/Ja-sonYun/yew-template-for-github-io),
  so it might be worth trying that.
- Work on the CSS to make it look at least decent. I've never
  actually done much CSS work, and this would be a good opportunity
  to gain some useful experience there.
- Using GraphQL for the queries. I've never actually done anything
  with GraphQL, but my understanding is that [GitHub has nice
  GraphQL support](https://docs.github.com/en/graphql),
  so this might be a good opportunity to explore
  that space. I can imagine situations where we might get info
  on a lot of repos (maybe more than 100) in an organization,
  and only need a fairly small subset of the details for each
  repository, which I think is exactly the kind of thing that
  GraphQL is good at. (I don't really know what I'm talking about,
  though. :stuck_out_tongue_winking_eye:) There are definitely
  GraphQL libraries and examples for Rust, but I haven't really
  looked at any of them.
- It might be interesting to build the same thing in a more
  "traditional" JS/TS framework as well, and it could be a nice
  excuse to try out something like [Svelte](https://svelte.dev) or 
  similar that I haven't used
  before. But I'm not sure that will happen, but you never know.
- Go through the full GitHub "[stuff you should do with your repo](<https://docs.github.com/en/communities>)"
  list, like have development guidelines and the like. I've seen
  things like that show up in the settings over the past several
  years, but I've never stepped back to take a look at what all
  is happening there or think about how that might apply in, e.g.,
  our course repositories.
