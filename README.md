# Decisions

## Philosophy

I chose to consider the csv as an adversarial data input, given that it represents interfacing with another system. I do a lot of validation at parse time, which allows
me to cut down on error handling down the line. This philosophy is most obvious in the fixedpoint module, in which the parsing has many many
lines of error handling, but the reverse operation has no error handling at all as it represents the end of the pipe where all errors should
have been handled at earlier points. I also chose to log all dropped transactions to stderr to ensure some follow up can be made with the ops team ;).
My choice of &'static str as error type did bite me there as I could not not log the precise tx id that caused the error. This was not corrected to fit 
in the 3h timeframe I gave myself.


## Choice of structures

### Fixed point

I chose to 'implement' my own fixed point integer, which is probably a controversial choice. The only reason I did that was to avoid
using floats to represent sensitive financial data and to be then subject to float precision errors. It might have been misguided in
that a f64 is probably big enough to never see such errors with the types of operations we are dealing with, but using i64 instead at least
guarantees no such floating point shenanigans. My fixed point implementation is also flawed in that it is not wrapped with a custom type (for cheap reuse of operators)
which brings forth some potential of misuse, specifically when interfacing with other systems or needing to print to string (for which I supplied a fixed_point_to_string)
function. The only reason I did not use a custom type was to develop faster in the 3h timeframe I was given.

The fixed point implementation can panic for reasons I considered most unlikely to save development time (3h timeframe). Notably, it will panic if the number container
a non-one amount of decimal points, or if the number contains any non-numeric characters, or if the number has a more than 4 digits past the decimal point precision. It would
be trivial to fix these issues given more time, or to switch to some other implementation. This behaviour is all documented as unit tests, so it would be easy to change the implementation
by changing the unit tests and correcting.

### Funds type

I chose to make available_funds, held_funds, and total_funds i64, which might be a surprise given that these funds were not explicitly said to be possibly negative.
I made that choice when reading the description of the dispute system. It seemed then obvious that a deposit could be disputed after a withdrawal, resulting in a negative 
balance for the affected account. To simplify arithmetic, and to represent an effectively u63 value, I decided to make all fixed point datatype i64. To prevent negative 
numbers in the csv, I check i64 > 0 at parse time.

## Resource usage

### Data structures

#### Memory Management

A lot of the decisions surrounding the representation of data in memory stem from the implication from the problem statement
that there might be in the order of 2^32 transactions. The main problem with that magnitude is the dispute system. Indeed, the dispute system backreferences transactions 
and that implies that the system must have some mechanism to retrieve past transactions effectively.
Given that transaction ids are not guaranteed to be sequential, or ordered, there is no effective way to traverse
the transactions file without parsing the whole thing again, taking in the order of 2^31 operations in average to find a given transaction id in the file.
The best way to manage this in my opinion with the given constraints would have been to use an on-disk database like SQLite, allowing low RAM usage
while keeping an index on past transactions which allows for decently quick access to past transactions on disk. Given that I had never used SQLite with rust, 
I did not opt for this strategy given the 3h time budget. Instead, I opted for a in-memory packed representation.

#### What I did to keep transactions small

First of all, given the problem statement, and the given definition of the various dispute operations, I realized that only deposits could
be disputed. In effect, this means that I only need to keep track of deposits in-memory instead of all transactions. Given that at a 2^32 magnitude, every
byte counts, I am using a deposit specialized structure to store the relevant data. In summary, I get rid of the TransactionType enum.

Another thing I did was to keep track of transactions under disputes using a HashSet, assuming that there would be a small amount of deposits under disputes
at any given time. This allows me to reduce the size of DepositRecords by a byte, which is ~4GB when scaled to 2^32 records.

Given the need to retrieve transactions by id effectively, you might think to use a HashMap to store transactions in memory. However, a HashMap
trades high memory usage for quick lookups, which we could not afford given the 2^32 magnitude of data. I opted instead to use a BTreeMap, which offers a log(n)
lookup complexity which, while being slower than O(1), should still be good for the (I assume) lower volume of disputes to other transactions, while offering a decently
packed representation in memory.

### Account representation

Weirdly enough, the given problem statement seemed to imply that I should keep track of total_funds. This property is obviously derived from available and held funds and therefore
I compute it instead on demand. Which pre-emptively gets rid of a ton of potential consistency bugs.

### CSV parsing

I stream records from the csv using a buffered reader provided by serde, this buffer size is configurable but I had no knowledge of the system it would run on to provide a good value.
It was therefore left to default size.

# Final note 

I had to make many sacrifices to keep my dev time under 3h, and had to forcibly stop myself from further polishing it. Hopefully the final result is still deemed acceptable :)