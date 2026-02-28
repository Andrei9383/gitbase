# Contributing to gitbase

First off, thank you for considering contributing to gitbase! It's people like you that make gitbase such a great tool.

## Where do I go from here?

If you've noticed a bug or have a feature request, make sure to check if there's already an issue open for it. If not, go ahead and open a new one!

## Fork & create a branch

If this is something you think you can fix, then fork gitbase and create a branch with a descriptive name.

A good branch name would be (where issue #325 is the ticket you're working on):

```sh
git checkout -b 325-add-awesome-feature
```

## Get the test suite running

Make sure you have Rust and Cargo installed.
Run the tests:

```sh
cargo test
```

## Implement your fix or feature

At this point, you're ready to make your changes. Feel free to ask for help; everyone is a beginner at first!

## Make a Pull Request

At this point, you should switch back to your master branch and make sure it's up to date with gitbase's master branch:

```sh
git remote add upstream git@github.com:USERNAME/gitbase.git
git checkout master
git pull upstream master
```

Then update your feature branch from your local copy of master, and push it!

```sh
git checkout 325-add-awesome-feature
git rebase master
git push --set-upstream origin 325-add-awesome-feature
```

Finally, go to GitHub and make a Pull Request!
