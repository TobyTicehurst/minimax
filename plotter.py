import csv
import matplotlib.pyplot as plt
import math


x = range(0, 50)

with open("log.csv", "r") as csvfile:
    reader = csv.reader(csvfile, delimiter=',')
    for row in reader:
        plt.plot(x, [math.log(int(i) + 1) for i in row])
    plt.show()