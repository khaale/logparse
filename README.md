# logparse
Parser for custom SSIS log format, written in Rust. Prints top aggregated timings of leaf tasks.

Input file(s) format:
```
Start...
Pre-execute package Package1 
2017-04-20T10:53:24.6607935+01:00 PRE EXECUTE Task1
2017-04-20T10:53:14.7607935+01:00 PRE EXECUTE Task2
2017-04-20T10:55:54.8607935+01:00 POST EXECUTE Task2
2017-04-20T10:55:24.9420381+01:00 POST EXECUTE Task1
Container Name       : Container1
End..

Start...
Pre-execute package Package2
2017-04-20T10:55:24.6607935+01:00 PRE EXECUTE Task1
2017-04-20T10:56:14.9420381+01:00 POST EXECUTE Task1
2017-04-20T10:57:55.6607935+01:00 PRE EXECUTE Task2
2017-04-20T10:58:25.9420381+01:00 POST EXECUTE Task2
Container Name       : Container1
End..
```

Output:
```
Package Name    Container Name  Task    Avg time (min)
Package1        Container1      Task2   2.67
Package2        Container1      Task1   0.83
Package2        Container1      Task2   0.50
```
