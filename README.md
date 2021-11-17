# Jimmy

Jimmy is a language-agnostic, Redis-backed job queue server with an HTTP interface and a focus on long running tasks.
It was forked from Ocypod with additional concerns in mind (see added features below).

# UI Client
[Sebastion](https://github.com/nidhhoggr/sebastion)

## Features added to Ocypod fork
* Ability to retry jobs
* Ability to specify which queue types get expired
* Adding file system persistence for failed job creation attempts [#1](https://github.com/joe-at-startupmedia/ocypod/issues/1)
* Adds an API method to reattempt a failed job attempt
* Adding lists for Compeleted and TimedOut (for better insight with constant-time lookup)
* Adds an API method to fetch a specific job from the queue by its id. (moves from queued to running) [sebastion#5](https://github.com/nidhhoggr/sebastion/issues/5)

## Base Ocypod Features

* simple setup - only requirement is Redis
* language agnostic - uses HTTP/JSON protocol, clients/workers can be
  implemented in any language
* long running jobs - handle jobs that may be running for hours/days,
  detect failure early using heartbeats
* simple HTTP interface - no complex binary protocols or client/worker logic
* flexible job metadata - allows for different patterns of use (e.g. progress
  tracking, partial results, etc.)
* job inspection - check the status of any jobs submitted to the system
* tagging - custom tags allow easy grouping and searching of related jobs
* automatic retries - re-queue jobs on failure or timeout
