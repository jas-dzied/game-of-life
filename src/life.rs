use bytemuck::{Pod, Zeroable};
use std::borrow::Cow;
use wgpu::{util::DeviceExt, BindGroup, Buffer, ComputePipeline, Device, Queue};

const WORKGROUP_SIZE: (u32, u32) = (16, 16);

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Params {
    pub width: u32,
    pub height: u32,
    lifetime: u32,
    a_rule_0: u32,
    a_rule_1: u32,
    a_rule_2: u32,
    a_rule_3: u32,
    a_rule_4: u32,
    a_rule_5: u32,
    a_rule_6: u32,
    a_rule_7: u32,
    a_rule_8: u32,
    d_rule_0: u32,
    d_rule_1: u32,
    d_rule_2: u32,
    d_rule_3: u32,
    d_rule_4: u32,
    d_rule_5: u32,
    d_rule_6: u32,
    d_rule_7: u32,
    d_rule_8: u32,
}

impl Params {
    pub fn new(
        width: u32,
        height: u32,
        lifetime: u32,
        alive_rules: [u32; 9],
        dead_rules: [u32; 9],
    ) -> Self {
        Self {
            width,
            height,
            lifetime,
            a_rule_0: alive_rules[0],
            a_rule_1: alive_rules[1],
            a_rule_2: alive_rules[2],
            a_rule_3: alive_rules[3],
            a_rule_4: alive_rules[4],
            a_rule_5: alive_rules[5],
            a_rule_6: alive_rules[6],
            a_rule_7: alive_rules[7],
            a_rule_8: alive_rules[8],
            d_rule_0: dead_rules[0],
            d_rule_1: dead_rules[1],
            d_rule_2: dead_rules[2],
            d_rule_3: dead_rules[3],
            d_rule_4: dead_rules[4],
            d_rule_5: dead_rules[5],
            d_rule_6: dead_rules[6],
            d_rule_7: dead_rules[7],
            d_rule_8: dead_rules[8],
        }
    }
}

pub struct Life {
    params: Params,
    data: Vec<u32>,
    compute_pipeline: ComputePipeline,
    compute_bind_group: BindGroup,
    compute_input_buffer: Buffer,
    compute_output_buffer: Buffer,
}

impl Life {
    pub fn new(
        data: Vec<u32>,
        params: Params,
        device: &Device,
        output_texture_view: &wgpu::TextureView,
    ) -> Self {
        let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("../shaders/life.wgsl"))),
        });
        let slice_size = data.len() * std::mem::size_of::<u32>();
        let size = slice_size as wgpu::BufferAddress;
        let compute_input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Input Buffer"),
            contents: bytemuck::cast_slice(&data),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });
        let compute_output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let compute_param_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("parameters buffer"),
            contents: bytemuck::bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("`compute pipeline`"),
            layout: None,
            module: &cs_module,
            entry_point: "main",
        });
        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute shader bind group"),
            layout: &compute_pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: compute_param_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: compute_input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: compute_output_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&output_texture_view),
                },
            ],
        });

        Self {
            params,
            data,
            compute_pipeline,
            compute_bind_group,
            compute_input_buffer,
            compute_output_buffer,
        }
    }

    pub async fn step(&mut self, device: &Device, queue: &Queue) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass =
                encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.compute_bind_group, &[]);
            cpass.insert_debug_marker("compute shader");

            let xgroups = self.params.width as u32 / WORKGROUP_SIZE.0;
            let ygroups = self.params.height as u32 / WORKGROUP_SIZE.0;

            cpass.dispatch_workgroups(xgroups, ygroups, 1);
            // Number of cells to run, the (x,y,z) size of item being processed
        }
        let slice_size = self.data.len() * std::mem::size_of::<u32>();
        let size = slice_size as wgpu::BufferAddress;

        // Sets adds copy operation to command encoder.
        // Will copy data from storage buffer on GPU to staging buffer on CPU.
        encoder.copy_buffer_to_buffer(
            &self.compute_output_buffer,
            0,
            &self.compute_input_buffer,
            0,
            size,
        );

        // Submits command encoder for processing
        queue.submit(Some(encoder.finish()));
    }
}
