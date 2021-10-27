import pickle
from numpy import ndarray

def check(val):                                                                                                                   
    if isinstance(val,ndarray ):
        return [float(x) for x in list(val.flatten())]
    else:
        return float(val)
    
if __name__ == "__main__":
    with open("domeseeing_PSSN.pickle","rb") as f:
        data = pickle.load(f)
    rs_data = [[{key:check(val) for (key,val) in x.items()}][0] for x in data]
    with open("domeseeing_PSSN.rs.pkl","wb") as f:
        pickle.dump(rs_data,f)

    
