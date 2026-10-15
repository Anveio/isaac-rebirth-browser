"""Stream a complete compact Native tuple object without retaining its rows."""
import gzip
import json


class JsonInput:
    def __init__(self, stream):
        self.stream, self.buffer, self.pos, self.eof = stream, '', 0, False
        self.decoder = json.JSONDecoder()
        self.maximum_value_bytes = 0

    def more(self):
        self.buffer = self.buffer[self.pos:]
        self.pos = 0
        block = self.stream.read(65536)
        self.buffer += block
        self.eof = not block

    def peek(self):
        while True:
            while self.pos < len(self.buffer) and self.buffer[self.pos].isspace():
                self.pos += 1
            if self.pos < len(self.buffer):
                return self.buffer[self.pos]
            if self.eof:
                return ''
            self.more()

    def expect(self, character):
        if self.peek() != character:
            raise ValueError('Expected JSON delimiter ' + character)
        self.pos += 1

    def value(self):
        self.peek()
        while True:
            try:
                value, end = self.decoder.raw_decode(self.buffer, self.pos)
            except json.JSONDecodeError:
                if self.eof:
                    raise
                self.more()
                continue
            if end == len(self.buffer):
                if not self.eof:
                    self.more()
                    continue
            elif self.buffer[end] not in ' \t\r\n:,]}':
                # raw_decode can accept the integer prefix of a fraction or
                # exponent split across input blocks. Wait for its delimiter.
                if not self.eof:
                    self.more()
                    continue
                raise ValueError('Invalid character after Native JSON value')
            self.maximum_value_bytes = max(self.maximum_value_bytes,
                len(self.buffer[self.pos:end].encode('utf8')))
            self.pos = end
            return value



def native_input(path, rows):
    """Stream a complete tuple object; rows are consumed incrementally by the caller."""
    with gzip.open(path, 'rt') as stream:
        reader = JsonInput(stream)
        reader.expect('{')
        result = {}
        while reader.peek() != '}':
            key = reader.value()
            if type(key) is not str or key in result:
                raise ValueError('Distinct actual JSON object key required')
            reader.expect(':')
            if key == 'rows':
                reader.expect('[')
                while reader.peek() != ']':
                    rows.append(reader.value())
                    if reader.peek() == ']':
                        break
                    reader.expect(',')
                    if reader.peek() == ']':
                        raise ValueError('Trailing comma in Native rows')
                reader.expect(']')
                result[key] = rows
            else:
                result[key] = reader.value()
            if reader.peek() == '}':
                break
            reader.expect(',')
            if reader.peek() == '}':
                raise ValueError('Trailing comma in Native tuple object')
        reader.expect('}')
        if reader.peek():
            raise ValueError('Trailing data after complete Native tuples')
        if result.get('rows') is not rows or not rows:
            raise ValueError('Actual nonempty Native rows array required')
        return result

