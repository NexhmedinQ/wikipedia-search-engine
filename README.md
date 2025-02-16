# Wikipedia Search Engine

### Intro
Hello to myself and any random person that opens my repo. This will be a minimal and probably dog ass implementation of a search engine but it should be a fun little learning exercise so strap yourself in and get ready for some deplorable Rust code. P.S the README will be treated like a dev log.
### Implementing a Basic Crawler - Part 1
So to actually make the search engine we need data (who would have thought). This initial commit will be a simple wikipedia crawler that will parse and send the tokens to the indexer which is yet to be implemented. I'll only be storing a hashmap of the visited URLs just so we don't run into a cycle.

Also I've made the decision to go easy on the Wikipedia servers so there's no use in doing any async processing or parallel calls to URLs in the queue. If I were to do that I'd maybe get my IP blocked so the crawler will just use a blocking client. So this initial commit won't have good or working code but I have been able to parse a Wikipedia page to a standard that is good enough for me. The next bit of work will be cleaning up the code and finishing off our simple crawler.

### Implementing the Inverted Index
Now we get to the fun stuff implementing a dynamic inverted index on disk. First I should mention that the search engine we're implementing is a simple TF - IDF engine without any fuzzy matching or the like at least for now. That means we can safely use a hash based data structure while the inverted index is kept in memory rather than a sort based one that would be required for performance benefits in the case of fuzzy matching e.g. matching "engin*" with a sort based structure means all the matching terms will be contiguous in memory so you can perform a binary search to find the first term and then perform a linear scan to find other terms. 

So we'll start off with a simple inverted index using Rust's inbuilt HashMap (may need to make custom HashMap in future depending on how easy it'll be to serialize the standard library's implementation). The HashMap will store the document frequency and a pointer to the postings list of the term. 

Now we get to the interesting bit of our simple inverted index. What kind of data structure should we use for the postings list? The way I see it there are two options that immediately come to mind:

1. Dynamically sized array (a Vec in Rust or vector in C++)
2. A linked list

however both of them have their own issues. The main issue with a Vec is that we're left with a lot of empty unused space (a lot of the space after resizing remains unused). A linked list doesn't suffer from this problem however it likely will consume even more memory. In our current posting list implementing we will store (docID, frequency) which can both be stored as 64 bit integers (or even as a u32). A linked list will require a 64 bit pointer to the next element meaning 33% to 50% of our memory usage will just be pointers. Also, linked lists don't make use of the CPU cache effectively since entries are not contiguous in memory. 

We can solve most of our problems by using a combination of these two data structures called an unrolled linked list. This is simply a linked list that stores multiple elements so naturally our implementation will be a linked list of arrays. This allows us to increase cache hits and decrease the amount of unused memory.  

I decided to double the number of entries after each linked list node to maximise cache hits so realistically I've now got a Vec but worse...
Might need to do some benchmark testing but I think I can just switch the unrolled linked list with a Vec or I can try have a constant small capacity on each node. 
