import sys
import matplotlib.pyplot as plt
filename = sys.argv[1]
outname = sys.argv[2]
assert outname.endswith(".eps")
data = []
with open(filename) as file:
    for line in file:
        line = line.strip()
        if line and line[0] not in 'nr':
            data.append([float(l) for l in line.split(";")])
        elif line and line[0] == 'r':
            data.append(line.split(";"))
plt.figure(figsize=(8,3))
#cmap = plt.get_cmap("tab10")

names = data[0]
nums = data[1:]
rhos = [row[0] for row in nums]
srpts = [row[1] for row in nums]
dots = ["-", "--", "-.", "--", "-", "-", "-", "-", "-"]
for col in range(1, len(names)):
    seks = [row[col] for row in nums]
    ratios = [1 - sek/srpt for (sek, srpt) in zip(seks, srpts)]

    name = names[col]
    plt.plot(rhos, ratios, label=name, linestyle=dots[col-1])
plt.xlabel("Load $\\rho$")
plt.ylabel("Improvement ratio")
plt.xlim(0.75,1.0)
plt.ylim(-0.02, 0.02)
#plt.legend(bbox_to_anchor=(1, 1), loc="upper left")
plt.legend(loc="lower left")
plt.savefig(outname, bbox_inches='tight')

