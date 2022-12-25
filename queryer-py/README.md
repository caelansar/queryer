### Setup python virtual env
```shell
python3 -m venv .env
source .env/bin/activate
pip install ipython maturin
```

### Build python module
```shell
maturin develop
```

### Usage
```python
In [1]: import queryer_py

In [2]: sql = "select * from file://../queryer-rs/examples/data.json"

In [3]: queryer_py.query(sql, "json")
Out[3]: '[{"name":"bb","age":22},{"name":"cc","age":18},{"name":"aa","age":15}]'

In [4]: queryer_py.query(sql, "csv")
Out[4]: 'name,age\nbb,22\ncc,18\naa,15\n'
```
