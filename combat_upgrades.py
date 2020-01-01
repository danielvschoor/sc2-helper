from simulator import *

class CombatUpgrades:
    def __init__(self, upgrades=None):
        self.upgrades = []
        if upgrades:
            for u in upgrades:
                self.upgrades.append(u)
        else:
            self.upgrades = [False for _ in range(90)]
        
    def hash(self):
        return hash(self.upgrades)
    
    def add(self, upgrade):
        self.upgrades[upgrade] = True
    
    def combine(self, other):
        self.upgrades |= other.upgrades
    
    def remove(self, other):
        self.upgrades &= other.upgrades
    
    def has_upgrade(self, upgrade):
        return self.upgrades[upgrade]
    
    
