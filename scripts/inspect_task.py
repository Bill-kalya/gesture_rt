from pathlib import Path
p = Path('models/hand_landmarker.task')
print('path', p)
print('exists', p.exists())
if not p.exists():
    raise SystemExit(1)
size = p.stat().st_size
print('size', size)
with p.open('rb') as f:
    hdr = f.read(8)
print('header hex', hdr.hex())
# check zip
if hdr.startswith(b'PK'):
    import zipfile
    with zipfile.ZipFile(p) as z:
        print('zip members:')
        for info in z.infolist():
            print(info.filename, info.file_size)
        # try to extract .tflite members
        for info in z.infolist():
            if info.filename.endswith('.tflite'):
                out = Path('models')/Path(info.filename).name
                print('extracting', info.filename, '->', out)
                z.extract(info, path='models')
                print('extracted', out)
else:
    data = p.read_bytes()
    # search for .tflite filename occurrences
    idxs = []
    for needle in [b'.tflite', b'TFL3']:
        i = data.find(needle)
        if i!=-1:
            idxs.append((needle.decode(errors='ignore'), i))
    if idxs:
        print('found signatures at offsets:', idxs)
    # search for embedded tflite by looking for flatbuffer magic 'TFL3' or file extension
    start = data.find(b'TFL3')
    if start!=-1:
        # try to dump from start to end into a file
        out = Path('models/hand_landmark_extracted.tflite')
        out.write_bytes(data[start:])
        print('wrote probable tflite to', out)
    else:
        # search for '.tflite' string; extract surrounding bytes
        pos = data.find(b'.tflite')
        if pos!=-1:
            # find preceding null-terminated filename
            fname_start = data.rfind(b'\x00', 0, pos) + 1
            fname = data[fname_start:pos+7].decode(errors='ignore')
            print('embedded filename', fname)
        else:
            print('no tflite signature found')
print('done')
