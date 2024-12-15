use bevy::{
    prelude::*,
    render::{
        render_graph::{self},
        render_resource::{ComputePassDescriptor, PipelineCache},
        renderer::RenderContext,
    },
};

use crate::{
    constants::*, parameters::ParamsUniform, pipeline::ComputePipelines, BindGroupSelection, GpuBufferBindGroups, ParamsChanged, ShaderConfigHolder
};

#[derive(Clone)]
pub enum ComputeNodeMode {
    Extract,
    Compute2D(usize),
    Compute1D(usize),
}

// #[derive(Default)]
pub struct ComputeNode {
    pub pipeline_index: usize,
    // pub is_final: bool,
    pub mode: ComputeNodeMode,
}



impl render_graph::Node for ComputeNode {
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipelines = world.resource::<ComputePipelines>();
        let bind_groups = world.resource::<GpuBufferBindGroups>();
        let encoder = render_context.command_encoder();
        let selectors = world.resource::<BindGroupSelection>();
        let shader_configurator = world.resource::<ShaderConfigHolder>();
        let changed = world.resource::<ParamsChanged>();

        if !changed.0{
            // println!("not changed");
            return Ok(());
        }
            
        // println!("changed");
        
        match self.mode {
            ComputeNodeMode::Extract => {
                if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipelines.final_pass) {
                    encoder.push_debug_group("Final pass");

                    {
                        let group = if selectors.final_pass == 0 {
                            &bind_groups.final_pass_a
                        } else {
                            &bind_groups.final_pass_b
                        };

                        let mut pass =
                            encoder.begin_compute_pass(&ComputePassDescriptor::default());
                        pass.set_bind_group(0, group, &[]);
                        pass.set_pipeline(pipeline);
                        pass.dispatch_workgroups(
                            ((BUFFER_LEN + 15) / 16) as u32,
                            ((BUFFER_LEN + 15) / 16) as u32,
                            1,
                        );
                    }
                    encoder.pop_debug_group();
                }
            }
            ComputeNodeMode::Compute2D(buffer_len) => {
                let pipeline_id = pipelines.pipeline_configs[self.pipeline_index];

                if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id) {
                    let iters = shader_configurator.shader_configs[self.pipeline_index].iterations;
                    // println!("iters: {}", iters);
                    for iteration in 0..iters {
                        encoder.push_debug_group(&format!(
                            "Compute pass {} iteration {}",
                            self.pipeline_index, iteration
                        ));

                        {
                            let node = self.pipeline_index as u32;
                            let selection = selectors.selectors[&node][iteration as usize];
                            let mut pass =
                                encoder.begin_compute_pass(&ComputePassDescriptor::default());
                            pass.set_bind_group(
                                0,
                                &bind_groups.bind_groups[selection as usize],
                                &[],
                            );
                            pass.set_pipeline(pipeline);
                            // println!("dispatching iteration {}", iteration);
                            pass.dispatch_workgroups(
                                ((buffer_len + 15) / 16) as u32,
                                ((buffer_len + 15) / 16) as u32,
                                1,
                            );
                        }
                        encoder.pop_debug_group();
                    }
                }
            } 
            ComputeNodeMode::Compute1D(buffer_len) => {
                let pipeline_id = pipelines.pipeline_configs[self.pipeline_index];

                if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id) {
                    let iters = shader_configurator.shader_configs[self.pipeline_index].iterations;

                    for iteration in 0..iters {
                        encoder.push_debug_group(&format!(
                            "Compute pass {} iteration {}",
                            self.pipeline_index, iteration
                        ));

                        {
                            let node = self.pipeline_index as u32;
                            let selection = selectors.selectors[&node][iteration as usize];
                            let mut pass =
                                encoder.begin_compute_pass(&ComputePassDescriptor::default());
                            pass.set_bind_group(
                                0,
                                &bind_groups.bind_groups[selection as usize],
                                &[],
                            );
                            pass.set_pipeline(pipeline);
                            pass.dispatch_workgroups(
                                ((buffer_len + 255) / 256) as u32,
                                1,
                                1,
                            );
                        }
                        encoder.pop_debug_group();
                    }
                }
            } 
            
            
            // if self.is_final {
              //     if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipelines.final_pass) {
              //         encoder.push_debug_group("Final pass");

              //         {
              //             let group = if selectors.final_pass == 0 {
              //                 &bind_groups.final_pass_a
              //             } else {
              //                 &bind_groups.final_pass_b
              //             };

              //             let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
              //             pass.set_bind_group(0, group, &[]);
              //             pass.set_pipeline(pipeline);
              //             pass.dispatch_workgroups(
              //                 ((BUFFER_LEN + 15) / 16) as u32,
              //                 ((BUFFER_LEN + 15) / 16) as u32,
              //                 1,
              //             );
              //         }
              //         encoder.pop_debug_group();
              //     }
              // } else {
              // let pipeline_id = pipelines.pipeline_configs[self.pipeline_index];

              // if let Some(pipeline) = pipeline_cache.get_compute_pipeline(pipeline_id) {
              //     let iters = shader_configurator.shader_configs[self.pipeline_index].iterations;

              //     for iteration in 0..iters {
              //         encoder.push_debug_group(&format!(
              //             "Compute pass {} iteration {}",
              //             self.pipeline_index, iteration
              //         ));

              //         {
              //             let node = self.pipeline_index as u32;
              //             let selection = selectors.selectors[&node][iteration as usize];
              //             let mut pass =
              //                 encoder.begin_compute_pass(&ComputePassDescriptor::default());
              //             pass.set_bind_group(0, &bind_groups.bind_groups[selection as usize], &[]);
              //             pass.set_pipeline(pipeline);
              //             pass.dispatch_workgroups(
              //                 ((BUFFER_LEN + 15) / 16) as u32,
              //                 ((BUFFER_LEN + 15) / 16) as u32,
              //                 1,
              //             );
              //         }
              //         encoder.pop_debug_group();
              //     }
              // }
        }

        Ok(())
    }
}
