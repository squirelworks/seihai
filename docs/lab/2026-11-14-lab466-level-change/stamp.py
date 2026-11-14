import sys, time
o = open(sys.argv[1], "a")
for line in sys.stdin:
    o.write("%.3f %s" % (time.time(), line)); o.flush()
