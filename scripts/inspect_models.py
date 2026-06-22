from pathlib import Path
import sys
print('inspect models')
for p in Path('models').glob('*'):
    if p.suffix == '.onnx':
        import onnx
        print('\nONNX:', p)
        model = onnx.load(str(p))
        print('  opset', model.opset_import)
        for inp in model.graph.input:
            shape = [dim.dim_value if dim.dim_value > 0 else '?' for dim in inp.type.tensor_type.shape.dim]
            print('  input', inp.name, inp.type.tensor_type.elem_type, shape)
        for out in model.graph.output:
            shape = [dim.dim_value if dim.dim_value > 0 else '?' for dim in out.type.tensor_type.shape.dim]
            print('  output', out.name, out.type.tensor_type.elem_type, shape)
    elif p.suffix == '.tflite':
        print('\nTFLITE:', p)
        try:
            import tflite
            data = p.read_bytes()
            model = tflite.Model.GetRootAsModel(data, 0)
            print('  tflite ok, subgraphs', model.SubgraphsLength())
            for si in range(model.SubgraphsLength()):
                sg = model.Subgraphs(si)
                print('   subgraph', si, sg.Name())
                for ii in range(sg.InputsLength()):
                    idx = sg.Inputs(ii)
                    tensor = sg.Tensors(idx)
                    shape = [tensor.Shape(j) for j in range(tensor.ShapeLength())]
                    print('    input', idx, tensor.Name(), shape, tensor.Type())
                for ii in range(sg.OutputsLength()):
                    idx = sg.Outputs(ii)
                    tensor = sg.Tensors(idx)
                    shape = [tensor.Shape(j) for j in range(tensor.ShapeLength())]
                    print('    output', idx, tensor.Name(), shape, tensor.Type())
        except Exception as e:
            print('  failed tflite inspect:', e)
    elif p.name == 'hand_landmarker.task':
        print('\nTASK:', p, 'size', p.stat().st_size)
print('\ndone')
