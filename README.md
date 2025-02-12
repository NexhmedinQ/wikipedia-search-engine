# Wikipedia Search Engine

### Intro
Hello to myself and any random person that opens my repo. This will be a minimal and probably dog ass implementation of a search engine but it should be a fun little learning exercise so strap yourself in and get ready for some deplorable Rust code. P.S the README will be treated like a dev log.
### Implementing a Basic Crawler - Part 1
So to actually make the search engine we need data (who would have thought). This initial commit will be a simple wikipedia crawler that will parse and send the tokens to the indexer which is yet to be implemented. I'll only be storing a hashmap of the visited URLs just so we don't run into a cycle.

Also I've made the decision to go easy on the Wikipedia servers so there's no use in doing any async processing or parallel calls to URLs in the queue. If I were to do that I'd maybe get my IP blocked so the crawler will just use a blocking client. So this initial commit won't have good or working code but I have been able to parse a Wikipedia page to a standard that is good enough for me. The next bit of work will be cleaning up the code and finishing off our simple crawler.