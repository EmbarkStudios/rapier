use super::{PositionSolver, VelocitySolver};
use crate::counters::Counters;
use crate::dynamics::solver::{
    AnyJointPositionConstraint, AnyJointVelocityConstraint, AnyPositionConstraint,
    AnyVelocityConstraint, SolverConstraints,
};
use crate::dynamics::{IntegrationParameters, JointGraphEdge, JointIndex, RigidBodySet};
use crate::geometry::{ContactManifold, ContactManifoldIndex};

pub struct IslandSolver {
    contact_constraints: SolverConstraints<AnyVelocityConstraint, AnyPositionConstraint>,
    joint_constraints: SolverConstraints<AnyJointVelocityConstraint, AnyJointPositionConstraint>,
    velocity_solver: VelocitySolver,
    position_solver: PositionSolver,
}

impl IslandSolver {
    pub fn new() -> Self {
        Self {
            contact_constraints: SolverConstraints::new(),
            joint_constraints: SolverConstraints::new(),
            velocity_solver: VelocitySolver::new(),
            position_solver: PositionSolver::new(),
        }
    }

    pub fn solve_island(
        &mut self,
        island_id: usize,
        counters: &mut Counters,
        params: &IntegrationParameters,
        bodies: &mut RigidBodySet,
        manifolds: &mut [&mut ContactManifold],
        manifold_indices: &[ContactManifoldIndex],
        joints: &mut [JointGraphEdge],
        joint_indices: &[JointIndex],
    ) {
        let has_constraints = manifold_indices.len() != 0 || joint_indices.len() != 0;

        if has_constraints {
            // Symplectic Euler: initialize contraints with the *old* positions.
            counters.solver.velocity_assembly_time.resume();
            self.contact_constraints
                .init(island_id, params, bodies, manifolds, manifold_indices);
            self.joint_constraints
                .init(island_id, params, bodies, joints, joint_indices);
            counters.solver.velocity_assembly_time.pause();

            // Symplectic Euler: move bodies using the *old* velocities.
            counters.solver.velocity_update_time.resume();
            bodies.foreach_active_island_body_mut_internal(island_id, |_, rb| {
                rb.integrate(params.dt)
            });
            counters.solver.velocity_update_time.pause();

            counters.solver.velocity_resolution_time.resume();
            self.velocity_solver.solve(
                island_id,
                params,
                bodies,
                manifolds,
                joints,
                &mut self.contact_constraints.velocity_constraints,
                &mut self.joint_constraints.velocity_constraints,
            );
            counters.solver.velocity_resolution_time.pause();

            counters.solver.position_resolution_time.resume();
            self.position_solver.solve(
                island_id,
                params,
                bodies,
                &self.contact_constraints.position_constraints,
                &self.joint_constraints.position_constraints,
            );
            counters.solver.position_resolution_time.pause();
        } else {
            counters.solver.velocity_update_time.resume();
            bodies.foreach_active_island_body_mut_internal(island_id, |_, rb| {
                rb.integrate(params.dt)
            });
            counters.solver.velocity_update_time.pause();
        }
    }
}
